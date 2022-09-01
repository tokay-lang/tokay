use super::iml::*;
use crate::value::Parselet;
use crate::vm::{Op, Program};
use crate::{RefValue, Value};
use indexmap::IndexMap;
use num::ToPrimitive;
use std::collections::HashMap;

/// The linker glues compiled intermediate program and finalized VM program together.
#[derive(Debug)]
pub(super) struct Linker {
    statics: IndexMap<ImlValue, Option<Parselet>>, // static values with optional final parselet replacement
}

impl Linker {
    pub fn new(main: ImlValue) -> Self {
        let mut statics = IndexMap::new();
        statics.insert(main, None);

        Linker { statics }
    }

    /** Registers an ImlValue in the Linker's statics map and returns its index.

    In case *value* already exists inside of the current statics, the existing index will be returned,
    otherwiese the value is cloned and put into the statics table. */
    pub fn register(&mut self, value: &ImlValue) -> usize {
        match self.statics.get_index_of(value) {
            None => self.statics.insert_full(value.clone(), None).0,
            Some(idx) => idx,
        }
    }

    /** Generates code for a value push. For several, oftenly used values, there exists a direct operation pendant,
    which makes storing the static value obsolete. Otherwise, *value* will be registered and a static load operation
    is returned. */
    pub fn push(&mut self, value: &ImlValue) -> Op {
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

    /** Turns the Linker and its intermediate values into a final VM program ready for execution.

    The finalization is done according to a grammar's point of view, as this is one of Tokays core features.
    This closure algorithm runs until no more changes on any parselet configurations regarding left-recursive
    and nullable parselet detection occurs.
    */
    pub fn finalize(mut self) -> Program {
        let mut finalize = Vec::new(); // list of consuming parselets required to be finalized

        // Loop until end of statics is reached
        let mut i = 0;

        while i < self.statics.len() {
            // Pick only intermediate parselets, other static values are directly moved
            let outer = {
                match self.statics.get_index(i).unwrap() {
                    (_, Some(_)) => unreachable!(), // may not exist!
                    (ImlValue::Parselet(parselet), None) => parselet.clone(),
                    _ => {
                        i += 1;
                        continue;
                    }
                }
            };

            let parselet = outer.borrow();

            // Memoize parselets required to be finalized (needs a general rework later...)
            if parselet.consuming {
                finalize.push(outer.clone());
            }

            let mut begin = Vec::new();
            let mut end = Vec::new();
            let mut body = Vec::new();

            parselet.begin.compile(&mut begin, &mut self);
            parselet.end.compile(&mut end, &mut self);
            parselet.body.compile(&mut body, &mut self);

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
                            // Copy parameter name
                            var_value.0.clone(),
                            // Register default value, if any
                            if let Some(value) = &var_value.1 {
                                Some(self.register(value))
                            } else {
                                None
                            },
                        )
                    })
                    .collect(),
                parselet.locals,
                begin,
                end,
                body,
            );

            *self.statics.get_index_mut(i).unwrap().1 = Some(parselet);
            i += 1;
        }

        // Now, start the closure algorithm with left-recursive and nullable configurations for all parselets
        // put into the finalize list.
        let mut configs = HashMap::new(); // hash-map of static-id and consuming configuration
        let mut changes = true;

        while changes {
            changes = false;

            for parselet in &finalize {
                let parselet = parselet.borrow_mut(); // parselet is locked for left-recursion detection
                changes |= parselet.finalize(&mut configs);
            }
        }

        /*
        for p in &finalize {
            let parselet = p.borrow();
            println!(
                "{} consuming={:?}",
                parselet.name.as_deref().unwrap_or("(unnamed)"),
                configs[&parselet.id()]
            );
        }
        */

        let statics: Vec<RefValue> = self
            .statics
            .into_iter()
            .map(|(iml, parselet)| {
                if let Some(mut parselet) = parselet {
                    if let ImlValue::Parselet(imlparselet) = iml {
                        //println!("{:?}", imlparselet.borrow().name);
                        parselet.consuming = configs
                            .get(&imlparselet.borrow().id())
                            .map_or(None, |config| Some(config.leftrec));
                    }

                    RefValue::from(parselet)
                } else {
                    iml.value()
                }
            })
            .collect();

        /*
        for (i, value) in statics.iter().enumerate() {
            println!("{} : {:?}", i, value.borrow());
        }
        */

        Program::new(statics)
    }
}
