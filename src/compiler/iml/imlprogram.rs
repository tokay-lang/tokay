//! ImlProgram glues ImlParselets, ImlOps and ImlValues together to produce a VM program.

use super::*;
use crate::value::Parselet;
use crate::vm::Program;
use crate::Error;
use crate::RefValue;
use indexmap::IndexMap;
use std::collections::HashMap;

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
                            match &var_value.1 {
                                ImlValue::Void => None,
                                value => Some(self.register(value)),
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
                        parselet.consuming = configs
                            .get(&imlparselet.borrow().id())
                            .map_or(None, |config| Some(config.leftrec));

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
}
