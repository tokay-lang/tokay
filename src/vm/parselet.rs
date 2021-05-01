use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::value::{Dict, Value};

/** Parselet is the conceptual building block of a Tokay program.

A parselet is like a function in ordinary programming languages, with the
exception that it can either be a snippet of parsing instructions combined with
semantic code, or just an ordinary function consisting of code and returning
values. In general, the destinction if a parselet is a just a function or "more"
can only be made by looking at the parselets instruction set.
*/

#[derive(Debug)]
pub struct Parselet {
    pub(crate) leftrec: bool, // Indicator if parselet is left-recursive. Determined on finalization.
    pub(crate) nullable: bool, // Indicator if parselet is nullable. Determined on finalization.
    pub(crate) consumes: bool, /* Indicator if parselet is consuming input.
                              This can both be set on creation and additionally is determined
                              during finalization. */
    pub(crate) silent: bool, // Indicator if parselet is silent. Results are discarded.
    signature: Vec<(String, Option<usize>)>, // Argument signature with default arguments
    locals: usize,           // Number of local variables present
    begin: Op,               // Begin-operations
    end: Op,                 // End-operations
    pub(crate) body: Op,     // Operations
}

impl Parselet {
    /// Creates a new parselet.
    pub fn new(
        signature: Vec<(String, Option<usize>)>,
        locals: usize,
        consumes: bool,
        silent: bool,
        begin: Op,
        end: Op,
        body: Op,
    ) -> Self {
        assert!(
            signature.len() <= locals,
            "signature may not be longer than locals..."
        );

        Self {
            leftrec: false,
            nullable: true,
            consumes,
            silent,
            signature,
            locals,
            begin,
            end,
            body,
        }
    }

    /// Turns parselet into a Value
    pub fn into_value(self) -> Value {
        Value::Parselet(Rc::new(RefCell::new(self)))
    }

    /** Run parselet on a given runtime.

    The main-parameter defines if the parselet behaves like a main loop or
    like subsequent parselet. */
    pub fn run(
        &self,
        runtime: &mut Runtime,
        args: usize,
        mut nargs: Option<Dict>,
        main: bool,
    ) -> Result<Accept, Reject> {
        let mut context = Context::new(
            runtime,
            self.locals,
            args,
            if main { self.locals } else { 0 }, // Hold runtime globals when this is main!
        );

        if !main {
            // Set remaining parameters to their defaults.
            for (i, arg) in (&self.signature[args..]).iter().enumerate() {
                let var = &mut context.runtime.stack[context.stack_start + args + i];
                if matches!(var, Capture::Empty) {
                    // Try to fill argument by named arguments dict
                    if let Some(ref mut nargs) = nargs {
                        if let Some(value) = nargs.remove(&arg.0) {
                            *var = Capture::from_value(value.clone());
                            continue;
                        }
                    }

                    if let Some(addr) = arg.1 {
                        *var = Capture::from_value(context.runtime.program.statics[addr].clone());
                        //println!("{} receives default {:?}", arg.0, var);
                    }
                }
            }
        } else {
            assert!(self.signature.len() == 0)
        }

        //println!("remaining {:?}", nargs);

        // Initialize parselet execution loop
        let mut results = Vec::new();
        let mut state = if let Op::Nop = self.begin {
            None
        } else {
            Some(true)
        };

        loop {
            let reader_start = context.runtime.reader.tell();

            let mut res = match state {
                // begin
                Some(true) => self.begin.run(&mut context),

                // end
                Some(false) => self.end.run(&mut context),

                // default
                None => self.body.run(&mut context),
            };

            /*
                In case this is the main parselet, try matching main as much
                as possible. This is only be the case when input is consumed.
            */
            if main {
                //println!("main res(1) = {:?}", res);
                res = match res {
                    Ok(Accept::Next) => Ok(Accept::Repeat(None)),

                    Ok(Accept::Return(value)) => Ok(Accept::Repeat(value)),

                    Ok(Accept::Push(capture)) => Ok(Accept::Repeat(match capture {
                        Capture::Range(range, _) => Some(
                            Value::String(context.runtime.reader.extract(&range)).into_refvalue(),
                        ),
                        Capture::Value(value, _) => Some(value),
                        _ => None,
                    })),
                    res => res,
                };
                //println!("main res(2) = {:?}", res);
            }

            // Evaluate result of parselet loop.
            match res {
                Ok(accept) => {
                    match accept {
                        Accept::Skip => return Ok(Accept::Next),

                        Accept::Return(value) => {
                            if let Some(value) = value {
                                if !self.silent {
                                    return Ok(Accept::Push(Capture::Value(value, 5)));
                                } else {
                                    return Ok(Accept::Push(Capture::Empty));
                                }
                            } else {
                                return Ok(Accept::Push(Capture::Empty));
                            }
                        }

                        Accept::Repeat(value) => {
                            if let Some(value) = value {
                                results.push(value);
                            }
                        }

                        Accept::Push(_) if self.silent => return Ok(Accept::Push(Capture::Empty)),

                        accept => {
                            if results.len() > 0 {
                                break;
                            }

                            return Ok(accept);
                        }
                    }

                    if main {
                        // In case no input was consumed in main loop, skip character
                        if state.is_none()
                            && context.runtime.reader.capture_from(&reader_start).len() == 0
                        {
                            context.runtime.reader.next();
                        }

                        // Clear input buffer
                        context.runtime.reader.commit();

                        // Clear memo table
                        context.runtime.memo.clear();
                    }
                }

                Err(reject) => {
                    match reject {
                        Reject::Skip => return Ok(Accept::Next),
                        Reject::Error(mut err) => {
                            // Patch source position on error, when no position already set
                            if let Some(source_offset) = context.source_offset {
                                err.patch_offset(source_offset);
                            }

                            return Err(Reject::Error(err));
                        }
                        Reject::Main if !main => return Err(Reject::Main),
                        _ => {}
                    }

                    // Skip character and reset reader start
                    if main && state.is_none() {
                        context.runtime.reader.next();
                        context.reader_start = context.runtime.reader.tell();
                    } else if results.len() > 0 && state.is_none() {
                        state = Some(false);
                        continue;
                    } else if state.is_none() {
                        return Err(reject);
                    }
                }
            }

            if let Some(false) = state {
                break;
            } else if context.runtime.reader.eof() {
                state = Some(false);
            } else {
                state = None;
            }
        }

        if results.len() > 1 {
            Ok(Accept::Push(Capture::Value(
                Value::List(Box::new(results)).into_refvalue(),
                5,
            )))
        } else if results.len() == 1 {
            Ok(Accept::Push(Capture::Value(results.pop().unwrap(), 5)))
        } else {
            Ok(Accept::Next)
        }
    }
}

impl std::cmp::PartialEq for Parselet {
    // It satisfies to just compare the parselet's memory address for equality
    fn eq(&self, other: &Self) -> bool {
        self as *const Parselet as usize == other as *const Parselet as usize
    }
}

impl std::hash::Hash for Parselet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self as *const Parselet as usize).hash(state);
    }
}

impl std::cmp::PartialOrd for Parselet {
    // It satisfies to just compare the parselet's memory address for equality
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let left = self as *const Parselet as usize;
        let right = other as *const Parselet as usize;

        left.partial_cmp(&right)
    }
}