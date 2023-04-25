//! ImlProgram glues ImlParselets, ImlOps and ImlValues together to produce a VM program.

use super::*;
use crate::value::Parselet;
use crate::vm::Program;
use crate::Error;
use crate::{Object, RefValue};
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub(in crate::compiler) struct ImlProgram {
    statics: IndexMap<ImlValue, Option<Parselet>>, // static values with optional final parselet replacement
    pub errors: Vec<Error>, // errors collected during finalization (at least these are unresolved symbols)
}

impl ImlProgram {
    pub fn new(main: ImlValue) -> Self {
        let mut statics = IndexMap::new();
        statics.insert(main, None);

        ImlProgram {
            statics,
            errors: Vec::new(),
        }
    }

    /** Registers an ImlValue in the ImlProgram's statics map and returns its index.

    In case *value* already exists inside of the current statics, the existing index will be returned,
    otherwiese the value is cloned and put into the statics table. */
    pub fn register(&mut self, value: &ImlValue) -> usize {
        if let ImlValue::Shared(value) = value {
            return self.register(&*value.borrow());
        }

        match self.statics.get_index_of(value) {
            None => self.statics.insert_full(value.clone(), None).0,
            Some(idx) => idx,
        }
    }

    /** Turns the ImlProgram and its intermediate values into a final VM program ready for execution.

    The finalization is done according to a grammar's point of view, as this is one of Tokays core features.
    This closure algorithm runs until no more changes on any parselet configurations regarding left-recursive
    and nullable parselet detection occurs.
    */
    pub fn compile(mut self) -> Result<Program, Vec<Error>> {
        let mut finalize = Vec::new(); // list of consuming parselets required to be finalized

        // Loop until end of statics is reached
        let mut i = 0;

        // self.statics grows inside of this while loop, therefore this condition.
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

            // Compile VM parselet from intermediate parselet
            let parselet = parselet.compile(&mut self);

            *self.statics.get_index_mut(i).unwrap().1 = Some(parselet);
            i += 1;
        }

        let leftrec = self.finalize(finalize);

        // Stop on any raised error
        if !self.errors.is_empty() {
            return Err(self.errors);
        }

        // Assemble all statics to be transferred into a Program
        let statics: Vec<RefValue> = self
            .statics
            .into_iter()
            .map(|(iml, parselet)| {
                if let Some(mut parselet) = parselet {
                    if let ImlValue::Parselet(imlparselet) = iml {
                        parselet.consuming = leftrec
                            .get(&imlparselet.borrow().id())
                            .map_or(None, |leftrec| Some(*leftrec));

                        //println!("{:?} => {:?}", imlparselet.borrow().name, parselet.consuming);
                    }

                    RefValue::from(parselet)
                } else {
                    iml.into_refvalue()
                }
            })
            .collect();

        /*
        for (i, value) in statics.iter().enumerate() {
            println!("{} : {:?}", i, value.borrow());
        }
        */

        Ok(Program::new(statics))
    }

    /** Internal function to finalize a program on a grammar's point of view.

    The finalization performs a closure algorithm on every parselet to detect

    - nullable parselets
    - left-recursive parselets

    until no more changes occur.

    It can only be run on a previously compiled program without any unresolved usages.
    */
    fn finalize(&mut self, parselets: Vec<ImlSharedParselet>) -> HashMap<usize, bool> {
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
        struct Consumable {
            leftrec: bool,
            nullable: bool,
        }

        // Finalize ImlValue
        fn finalize_value(
            value: &ImlValue,
            visited: &mut HashSet<usize>,
            configs: &mut HashMap<usize, Consumable>,
        ) -> Option<Consumable> {
            match value {
                ImlValue::Shared(value) => finalize_value(&*value.borrow(), visited, configs),
                ImlValue::Parselet(parselet) => {
                    match parselet.try_borrow() {
                        // In case the parselet cannot be borrowed, it is left-recursive!
                        Err(_) => Some(Consumable {
                            leftrec: true,
                            nullable: false,
                        }),
                        // Otherwise dive into this parselet...
                        Ok(parselet) => finalize_parselet(&parselet, visited, configs),
                    }
                }
                ImlValue::Value(callee) => {
                    if callee.is_consuming() {
                        //println!("{:?} called, which is nullable={:?}", callee, callee.is_nullable());
                        Some(Consumable {
                            leftrec: false,
                            nullable: callee.is_nullable(),
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }

        // Finalize ImlOp
        fn finalize_op(
            op: &ImlOp,
            visited: &mut HashSet<usize>,
            configs: &mut HashMap<usize, Consumable>,
        ) -> Option<Consumable> {
            match op {
                ImlOp::Call { target, .. } => finalize_value(target, visited, configs),
                ImlOp::Alt { alts } => {
                    let mut leftrec = false;
                    let mut nullable = false;
                    let mut consumes = false;

                    for alt in alts {
                        if let Some(consumable) = finalize_op(alt, visited, configs) {
                            leftrec |= consumable.leftrec;
                            nullable |= consumable.nullable;
                            consumes = true;
                        }
                    }

                    if consumes {
                        Some(Consumable { leftrec, nullable })
                    } else {
                        None
                    }
                }
                ImlOp::Seq { seq, .. } => {
                    let mut leftrec = false;
                    let mut nullable = true;
                    let mut consumes = false;

                    for item in seq {
                        if !nullable {
                            break;
                        }

                        if let Some(consumable) = finalize_op(item, visited, configs) {
                            leftrec |= consumable.leftrec;
                            nullable = consumable.nullable;
                            consumes = true;
                        }
                    }

                    if consumes {
                        Some(Consumable { leftrec, nullable })
                    } else {
                        None
                    }
                }
                ImlOp::If { then, else_, .. } => {
                    let then = finalize_op(then, visited, configs);

                    if let Some(else_) = finalize_op(else_, visited, configs) {
                        if let Some(then) = then {
                            Some(Consumable {
                                leftrec: then.leftrec || else_.leftrec,
                                nullable: then.nullable || else_.nullable,
                            })
                        } else {
                            Some(else_)
                        }
                    } else {
                        then
                    }
                }
                ImlOp::Loop {
                    initial,
                    condition,
                    body,
                    ..
                } => {
                    let mut ret: Option<Consumable> = None;

                    for part in [initial, condition, body] {
                        let part = finalize_op(part, visited, configs);

                        if let Some(part) = part {
                            ret = if let Some(ret) = ret {
                                Some(Consumable {
                                    leftrec: ret.leftrec || part.leftrec,
                                    nullable: ret.nullable || part.nullable,
                                })
                            } else {
                                Some(part)
                            }
                        }
                    }

                    ret
                }

                // DEPRECATED BELOW!!!
                ImlOp::Expect { body, .. } => finalize_op(body, visited, configs),
                ImlOp::Not { body } | ImlOp::Peek { body } => finalize_op(body, visited, configs),
                ImlOp::Repeat { body, min, .. } => {
                    if let Some(consumable) = finalize_op(body, visited, configs) {
                        if *min == 0 {
                            Some(Consumable {
                                leftrec: consumable.leftrec,
                                nullable: true,
                            })
                        } else {
                            Some(consumable)
                        }
                    } else {
                        None
                    }
                }

                // default case
                _ => None,
            }
        }

        // Finalize ImlParselet
        fn finalize_parselet(
            parselet: &ImlParselet,
            visited: &mut HashSet<usize>,
            configs: &mut HashMap<usize, Consumable>,
        ) -> Option<Consumable> {
            // ... only if it's generally flagged to be consuming.
            if !parselet.consuming {
                return None;
            }

            let id = parselet.id();

            if visited.contains(&id) {
                Some(Consumable {
                    leftrec: false,
                    nullable: configs[&id].nullable,
                })
            } else {
                visited.insert(id);

                if !configs.contains_key(&id) {
                    configs.insert(
                        id,
                        Consumable {
                            leftrec: false,
                            nullable: false,
                        },
                    );
                }

                for part in [&parselet.begin, &parselet.body, &parselet.end] {
                    if let Some(result) = finalize_op(part, visited, configs) {
                        if configs[&id] < result {
                            configs.insert(id, result);
                        }
                    }
                }

                visited.remove(&id);
                Some(Consumable {
                    leftrec: false,
                    nullable: configs[&id].nullable,
                })
            }
        }

        // Now, start the closure algorithm with left-recursive and nullable configurations for all parselets
        // put into the finalize list.
        let mut changes = true;
        let mut configs = HashMap::new();

        while changes {
            changes = false;

            for parselet in &parselets {
                let parselet = parselet.borrow_mut(); // parselet is locked for left-recursion detection
                changes = finalize_parselet(&*parselet, &mut HashSet::new(), &mut configs)
                    > configs.get(&parselet.id()).cloned();
            }
        }

        /*
        for parselet in parselets {
            let parselet = parselet.borrow();
            println!(
                "{} consuming={:?}",
                parselet.name.as_deref().unwrap_or("(unnamed)"),
                configs[&parselet.id()]
            );
        }
        */

        configs.into_iter().map(|(k, v)| (k, v.leftrec)).collect()
    }
}
