use super::iml::*;
use crate::value::Parselet;
use crate::vm::{Op, Program};
use crate::{RefValue, Value};
use indexmap::{IndexMap, IndexSet};
use num::ToPrimitive;
use std::cell::RefCell;
use std::rc::Rc;

/// The linker glues compiled intermediate program and finalized VM program together.
#[derive(Debug)]
pub(super) struct Linker {
    statics: IndexMap<ImlValue, Option<Parselet>>, // static values with optional replacement
}

impl Linker {
    pub fn new(main: ImlValue) -> Self {
        let mut statics = IndexMap::new();
        statics.insert(main, None);

        Linker { statics }
    }

    pub fn register(&mut self, value: &ImlValue) -> usize {
        match self.statics.get_index_of(value) {
            None => self.statics.insert_full(value.clone(), None).0,
            Some(idx) => idx,
        }
    }

    pub fn push(&mut self, value: &ImlValue) -> Op {
        // Primary value pushes can directly be made by specific VM commands
        if let ImlValue::Value(value) = value {
            match &*value.borrow() {
                Value::Void => return Op::PushVoid,
                Value::Null => return Op::PushNull,
                Value::True => return Op::PushTrue,
                Value::False => return Op::PushFalse,
                Value::Int(i) => match i.to_i64() {
                    Some(0) => return Op::Push0,
                    Some(1) => return Op::Push1,
                    _ => {}
                },
                _ => {}
            }
        }

        Op::LoadStatic(self.register(value))
    }

    pub fn finalize(mut self) -> Program {
        let mut i = 0;
        let mut finalize = Vec::new();

        // Loop until end of statics is reached
        while i < self.statics.len() {
            // Pick only intermediate parselets, ignore any other static value
            let p = {
                match self.statics.get_index(i).unwrap() {
                    (_, Some(_)) => unreachable!(), // may not exist!
                    (ImlValue::Parselet(parselet), None) => parselet.clone(),
                    _ => {
                        i += 1;
                        continue;
                    }
                }
            };

            let parselet = p.borrow();

            // Memoize parselets required to be finalized (needs a general rework later...)
            if parselet.consuming.is_some() {
                finalize.push(p.clone());
            }

            // Compile parselet from intermediate parselet
            let parselet = Parselet::new(
                parselet.name.clone(),
                None,
                parselet.severity,
                parselet
                    .signature
                    .iter()
                    .map(|var_value| {
                        (
                            var_value.0.clone(),
                            if let Some(value) = &var_value.1 {
                                Some(self.register(value))
                            } else {
                                None
                            },
                        )
                    })
                    .collect(),
                parselet.locals,
                parselet.begin.compile(&mut self),
                parselet.end.compile(&mut self),
                parselet.body.compile(&mut self),
            );

            *self.statics.get_index_mut(i).unwrap().1 = Some(parselet);
            i += 1;
        }

        let mut cycles = 0;
        let mut changes = true;

        while changes {
            changes = false;

            for parselet in &finalize {
                let mut parselet = parselet.borrow_mut();

                let closure = parselet.body.finalize(&mut IndexSet::new());
                //println!("closure on {:?} is {:?}", parselet.name, parselet.consuming);
                //println!("--> returns {:?}", closure);

                match (&mut parselet.consuming, closure) {
                    (Some(consuming), Some(closure)) => {
                        if *consuming < closure {
                            //println!("--> updating");
                            *consuming = closure;
                            changes = true;
                        }
                    }
                    _ => {}
                }
            }

            //println!("---\nClosure cycle {}", cycles);
            //let _ = io::stdin().read(&mut [0u8]).unwrap();

            cycles += 1;
        }

        for p in &finalize {
            let parselet = p.borrow();
            println!(
                "{} consuming={:?}",
                parselet.name.as_deref().unwrap_or("(unnamed)"),
                parselet.consuming
            );
        }

        Program::new(
            self.statics
                .into_iter()
                .map(|(iml, parselet)| {
                    if let Some(mut parselet) = parselet {
                        if let ImlValue::Parselet(imlparselet) = iml {
                            if let Some(consuming) = &imlparselet.borrow().consuming {
                                parselet.consuming = Some(consuming.leftrec);
                            }
                        }

                        RefValue::from(parselet)
                    } else {
                        iml.value()
                    }
                })
                .collect(),
        )
    }
}
