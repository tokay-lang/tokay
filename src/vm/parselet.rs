use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::error::Error;
use crate::value::{Dict, Value};

/** Parselet is the conceptual building block of a Tokay program.

A parselet is like a function in ordinary programming languages, with the
exception that it can either be a snippet of parsing instructions combined with
semantic code, or just an ordinary function consisting of code and returning
values. The destinction if a parselet represents just a function or a parselet is
done by the consuming-flag, which is determined by use of static tokens, parselets
and consuming builtins.

Parselets support static program constructs being left-recursive, and extend
the generated parse tree automatically until no more input can be consumed.
*/

#[derive(Debug)]
pub struct Parselet {
    pub(crate) leftrec: bool, // Indicator if parselet is left-recursive. Determined on finalization.
    pub(crate) nullable: bool, // Indicator if parselet is nullable. Determined on finalization.
    pub(crate) consuming: bool, /* Indicator if parselet is consuming input.
                              This can both be set on creation and additionally is determined
                              during finalization. */
    pub(crate) silent: bool, // Indicator if parselet is silent. Results are discarded.
    pub(crate) name: Option<String>, // Parselet's name from source (for debugging)
    signature: Vec<(String, Option<usize>)>, // Argument signature with default arguments
    pub(crate) locals: usize, // Number of local variables present
    begin: Op,               // Begin-operations
    end: Op,                 // End-operations
    pub(crate) body: Op,     // Operations
}

impl Parselet {
    /// Creates a new parselet.
    pub fn new(
        name: Option<String>,
        signature: Vec<(String, Option<usize>)>,
        locals: usize,
        consuming: bool,
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
            name,
            leftrec: false,
            nullable: true,
            consuming,
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

    fn resolve(&mut self, usages: &mut Vec<Vec<Op>>) {
        self.begin.resolve(usages);
        self.end.resolve(usages);
        self.body.resolve(usages);
    }

    /**
    Finalize the program by resolving any unresolved usages and
    according to a grammar's point of view;
    This closure algorithm runs until no more changes on any
    parselet configuration occurs.

    The algorithm detects correct flagging fore nullable and
    left-recursive for any consuming parselet.

    It requires all parselets consuming input to be known before
    the finalization phase. Normally, this is already known due
    to Tokays identifier classification.

    Maybe there will be a better method for this detection in
    future.
    */
    pub(crate) fn finalize(mut usages: Vec<Vec<Op>>, statics: &Vec<RefValue>) -> usize {
        let mut changes = true;
        let mut loops = 0;

        while changes {
            changes = false;

            for i in 0..statics.len() {
                if let Value::Parselet(parselet) = &*statics[i].borrow() {
                    let mut parselet = parselet.borrow_mut();

                    if loops == 0 {
                        parselet.resolve(&mut usages);
                    }

                    if !parselet.consuming {
                        continue;
                    }

                    let mut stack = vec![(i, parselet.nullable)];
                    if let Some((leftrec, nullable)) = parselet.body.finalize(statics, &mut stack) {
                        if parselet.leftrec != leftrec {
                            parselet.leftrec = leftrec;
                            changes = true;
                        }

                        if parselet.nullable != nullable {
                            parselet.nullable = nullable;
                            changes = true;
                        }

                        if !parselet.consuming {
                            parselet.consuming = true;
                            changes = true;
                        }
                    }
                }
            }

            loops += 1;
        }

        /*
        for i in 0..statics.len() {
            if let Value::Parselet(parselet) = &*statics[i].borrow() {
                let parselet = parselet.borrow();
                println!(
                    "{} consuming={} leftrec={} nullable={}",
                    parselet.name.as_deref().unwrap_or("(unnamed)"),
                    parselet.consuming,
                    parselet.leftrec,
                    parselet.nullable
                );
            }
        }

        println!("Finalization finished after {} loops", loops);
        */

        loops
    }

    // Checks if parselet is callable with or without arguments
    pub(crate) fn is_callable(&self, with_arguments: bool) -> bool {
        // Either without arguments and signature is empty or all arguments have default values
        (!with_arguments && (self.signature.len() == 0 || self.signature.iter().all(|arg| arg.1.is_some())))
        // or with arguments and signature exists
            || (with_arguments && self.signature.len() > 0)
    }

    fn _run(&self, context: &mut Context, main: bool) -> Result<Accept, Reject> {
        // Initialize parselet execution loop
        let mut results = Vec::new();
        let mut state = if let Op::Nop = self.begin {
            None
        } else {
            Some(true)
        };

        let result = loop {
            let reader_start = context.runtime.reader.tell();
            let mut result = match state {
                // begin
                Some(true) => self.begin.run(context),

                // end
                Some(false) => self.end.run(context),

                // default
                None => self.body.run(context),
            };

            /*
                In case this is the main parselet, try matching main as much
                as possible. This is only the case when input is consumed.
            */
            if main {
                //println!("main result(1) = {:?}", result);
                result = match result {
                    Ok(Accept::Next) => Ok(Accept::Repeat(None)),

                    Ok(Accept::Return(value)) => Ok(Accept::Repeat(value)),

                    Ok(Accept::Push(capture)) => Ok(Accept::Repeat(match capture {
                        Capture::Range(range, _) => Some(
                            Value::String(context.runtime.reader.extract(&range)).into_refvalue(),
                        ),
                        Capture::Value(value, _) => Some(value),
                        _ => None,
                    })),
                    result => result,
                };
                //println!("main result(2) = {:?}", result);
            }

            // Evaluate result of parselet loop.
            match result {
                Ok(accept) => {
                    match accept {
                        Accept::Skip => break Some(Ok(Accept::Next)),

                        Accept::Return(value) => {
                            if let Some(value) = value {
                                if !self.silent {
                                    break Some(Ok(Accept::Push(Capture::Value(value, 5))));
                                } else {
                                    break Some(Ok(Accept::Push(Capture::Empty)));
                                }
                            } else {
                                break Some(Ok(Accept::Push(Capture::Empty)));
                            }
                        }

                        Accept::Repeat(value) => {
                            if let Some(value) = value {
                                results.push(value);
                            }
                        }

                        Accept::Push(_) if self.silent => {
                            break Some(Ok(Accept::Push(Capture::Empty)))
                        }

                        accept => {
                            if results.len() > 0 {
                                break None;
                            }

                            break Some(Ok(accept));
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
                        Reject::Skip => break Some(Ok(Accept::Next)),
                        Reject::Error(mut err) => {
                            // Patch source position on error, when no position already set
                            if let Some(source_offset) = context.source_offset {
                                err.patch_offset(source_offset);
                            }

                            break Some(Err(Reject::Error(err)));
                        }
                        Reject::Main if !main => break Some(Err(Reject::Main)),
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
                        break Some(Err(reject));
                    }
                }
            }

            if let Some(false) = state {
                break None;
            } else if context.runtime.reader.eof() {
                state = Some(false);
            } else {
                state = None;
            }

            // Reset capture stack for loop repeat ($0 must be kept alive)
            context.runtime.stack.truncate(context.capture_start + 1);
        };

        result.unwrap_or_else(|| {
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
        })
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
        // Check for a previously memoized result in memo table
        let id = self as *const Parselet as usize;

        if !main && self.consuming {
            // Get unique parselet id from memory address
            let reader_start = runtime.reader.tell();

            if let Some((reader_end, result)) = runtime.memo.get(&(reader_start.offset, id)) {
                runtime.reader.reset(*reader_end);
                return result.clone();
            }
        }

        // If not, start a new context.
        let mut context = Context::new(
            runtime,
            self,
            args,
            if main { self.locals } else { 0 }, // Hold runtime globals when this is main!
        );

        if !main {
            // Check for provided argument count bounds first
            // todo: Not executed when *args-catchall is implemented
            if args > self.signature.len() {
                return Error::new(
                    None,
                    format!(
                        "Too many parameters, {} possible, {} provided",
                        self.signature.len(),
                        args
                    ),
                )
                .into_reject();
            }

            // Set remaining parameters to their defaults
            for (i, arg) in (&self.signature[args..]).iter().enumerate() {
                let var = &mut context.runtime.stack[context.stack_start + args + i];
                //println!("{} {:?} {:?}", i, arg, var);
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
                        continue;
                    }

                    return Error::new(None, format!("Parameter '{}' required", arg.0))
                        .into_reject();
                }
            }

            // Check for remaining nargs
            // todo: Not executed when **nargs-catchall is implemented
            if let Some(nargs) = nargs {
                if let Some(narg) = nargs.iter().next() {
                    return Error::new(
                        None,
                        format!("Parameter '{}' provided to call but not used", narg.0),
                    )
                    .into_reject();
                }
            }
        } else
        /* main */
        {
            assert!(self.signature.len() == 0)
        }

        // Initialize locals
        for i in 0..self.locals {
            if let Capture::Empty = context.runtime.stack[context.stack_start + i] {
                context.runtime.stack[context.stack_start + i] =
                    Capture::Value(Value::Void.into_refvalue(), 10);
            }
        }

        //println!("remaining {:?}", nargs);

        // Check for an existing memo-entry, and return it in case of a match
        if !main && self.consuming {
            if self.leftrec {
                /*
                println!(
                    "--- {} @ {} ---",
                    self.name.as_deref().unwrap_or("(unnamed)"),
                    context.reader_start.offset
                );
                */

                // Left-recursive parselets are called in a loop until no more input
                // is consumed.
                let mut reader_end = context.reader_start;
                let mut result = Err(Reject::Next);

                // Insert a fake memo entry to avoid endless recursion
                context.runtime.memo.insert(
                    (context.reader_start.offset, id),
                    (reader_end, result.clone()),
                );

                loop {
                    let loop_result = self._run(&mut context, main);

                    match loop_result {
                        // Hard reject
                        Err(Reject::Main) | Err(Reject::Error(_)) => {
                            result = loop_result;
                            break;
                        }

                        // Soft reject
                        Err(_) => break,

                        _ => {}
                    }

                    let loop_end = context.runtime.reader.tell();

                    // Stop when no more input was consumed
                    if loop_end.offset <= reader_end.offset {
                        break;
                    }

                    result = loop_result;
                    reader_end = loop_end;

                    // Save intermediate result in memo table
                    context.runtime.memo.insert(
                        (context.reader_start.offset, id),
                        (reader_end, result.clone()),
                    );

                    // Reset reader & stack
                    context.runtime.reader.reset(context.reader_start);
                    context.runtime.stack.truncate(context.stack_start);
                    context
                        .runtime
                        .stack
                        .resize(context.capture_start + 1, Capture::Empty);
                }

                context.runtime.reader.reset(reader_end);

                return result;
            }
        }

        let result = self._run(&mut context, main);

        if !main && self.consuming {
            context.runtime.memo.insert(
                (context.reader_start.offset, id),
                (context.runtime.reader.tell(), result.clone()),
            );
        }

        result
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
