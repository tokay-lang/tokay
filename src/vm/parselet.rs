use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::value::{RefValue, Value};

/** Parselet is the conceptual building block of a Tokay program.

A parselet is like a function in ordinary programming languages, with the
exception that it can either be a snippet of parsing instructions combined with
semantic code, or just an ordinary function consisting of code and returning
values. In general, the destinction if a parselet is a just a function or "more"
can only be made by looking at the parselets instruction set.
*/

#[derive(Debug)]
pub struct Parselet {
    pub(crate) leftrec: bool,
    pub(crate) nullable: bool,
    silent: bool,
    signature: Vec<(String, Option<usize>)>,
    locals: usize,
    begin: Op,
    end: Op,
    pub(crate) body: Op,
}

impl Parselet {
    // Creates a new standard parselet.
    pub fn new(body: Op, locals: usize, begin: Op, end: Op) -> Self {
        Self {
            leftrec: false,
            nullable: true,
            silent: false,
            signature: Vec::new(),
            locals,
            begin,
            end,
            body,
        }
    }

    /// Creates a new silent parselet, which does always return Capture::Empty
    pub fn new_silent(body: Op, locals: usize, begin: Op, end: Op) -> Self {
        Self {
            leftrec: false,
            nullable: true,
            silent: true,
            signature: Vec::new(),
            locals,
            begin,
            end,
            body,
        }
    }

    // Turn parselet into RefValue
    pub fn into_refvalue(self) -> RefValue {
        Value::Parselet(Rc::new(RefCell::new(self))).into_ref()
    }

    /** Run parselet on runtime.

    The main-parameter defines if the parselet behaves like a main loop or
    like subsequent parselet. */
    pub fn run(&self, runtime: &mut Runtime, main: bool) -> Result<Accept, Reject> {
        let mut context = Context::new(runtime, self.locals);
        let mut results = Vec::new();
        let mut state = if let Op::Nop = self.begin { None } else { Some(true) };

        loop {
            let reader_start = context.runtime.reader.tell();
            println!("{:?} @ {:?}", state, reader_start);

            let mut res = match state {
                // begin
                Some(true) => self.begin.run(&mut context),

                // end
                Some(false) => self.end.run(&mut context),

                // default
                None => self.body.run(&mut context)
            };

            println!("{:?} => {:?}", state, res);

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
                        Capture::Range(range, _) => {
                            Some(Value::String(context.runtime.reader.extract(&range)).into_ref())
                        }
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

                            return Ok(accept)
                        },
                    }

                    // In case that no more input was consumed, stop here.
                    if main && state.is_none() && reader_start == context.runtime.reader.tell() {
                        context.runtime.reader.next();
                    }
                }

                Err(reject) => {
                    match reject {
                        Reject::Error(err) => return Err(Reject::Error(err)),
                        Reject::Main if !main => return Err(Reject::Main),
                        _ => {}
                    }

                    // Skip character
                    if main && state.is_none() {
                        context.runtime.reader.next();
                    } else if results.len() > 0 && state.is_none() {
                        state = Some(false);
                        continue;
                    }
                    else if state.is_none() {
                        return Err(reject);
                    }
                }
            }

            if let Some(false) = state {
                break;
            }
            else if context.runtime.reader.eof() {
                state = Some(false);
            }
            else {
                state = None;
            }
        }

        if results.len() > 1 {
            Ok(Accept::Push(Capture::Value(
                Value::List(Box::new(results)).into_ref(),
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
