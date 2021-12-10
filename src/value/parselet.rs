//! Parselet object represents a callable, user-defined function.

use std::cell::RefCell;
use std::rc::Rc;

use super::{Dict, Object, Value};
use crate::compiler::ast;
use crate::error::Error;
use crate::vm::*;

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
    pub(crate) name: Option<String>, // Parselet's name from source (for debugging)
    pub(crate) consuming: Option<bool>, // Indicator for consuming & left-recursion
    pub(crate) severity: u8,         // Capture push severity
    signature: Vec<(String, Option<usize>)>, // Argument signature with default arguments
    pub(crate) locals: usize,        // Number of local variables present
    begin: Vec<Op>,                  // Begin-operations
    end: Vec<Op>,                    // End-operations
    body: Vec<Op>,                   // Operations
}

impl Parselet {
    /// Creates a new parselet.
    pub fn new(
        name: Option<String>,
        consuming: Option<bool>,
        severity: u8,
        signature: Vec<(String, Option<usize>)>,
        locals: usize,
        begin: Vec<Op>,
        end: Vec<Op>,
        body: Vec<Op>,
    ) -> Self {
        assert!(
            signature.len() <= locals,
            "signature may not be longer than locals..."
        );

        Self {
            name,
            consuming,
            severity,
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

    fn _run(&self, context: &mut Context, main: bool) -> Result<Accept, Reject> {
        // Initialize parselet execution loop
        let mut first = self.begin.len() > 0;
        let mut results = Vec::new();
        let mut state = if self.begin.len() == 0 {
            None
        } else {
            Some(true)
        };

        // Debugging
        let mut debug = context.runtime.debug;
        if debug < 3 {
            if let (Ok(inspect), Some(name)) = (std::env::var("TOKAY_INSPECT"), self.name.as_ref())
            {
                if inspect.find(name).is_some() {
                    debug = 6;
                }
            }
        }

        let result = loop {
            let ops = match state {
                // begin
                Some(true) => &self.begin,

                // end
                Some(false) => &self.end,

                // default
                None => &self.body,
            };

            let mut result = Op::execute(ops, context, debug);

            // if main {
            //     println!("state = {:?} result = {:?}", state, result);
            // }

            /*
                In case this is the main parselet, try matching main as much
                as possible. This is only the case when input is consumed.
            */
            if main {
                //println!("main result(1) = {:#?}", result);
                result = match result {
                    Ok(Accept::Next) => Ok(Accept::Repeat(None)),

                    Ok(Accept::Return(value)) => Ok(Accept::Repeat(value)),

                    Ok(Accept::Push(capture)) => Ok(Accept::Repeat(match capture {
                        Capture::Range(range, ..) => {
                            Some(Value::String(context.runtime.reader.extract(&range)).into())
                        }
                        Capture::Value(value, ..) => Some(value),
                        _ => None,
                    })),
                    result => result,
                };
                //println!("main result(2) = {:#?}", result);
            }

            // if main {
            //     println!("state = {:?} result = {:?}", state, result);
            // }

            // Evaluate result of parselet loop.
            match result {
                Ok(accept) => {
                    match accept {
                        Accept::Hold => break Some(Ok(Accept::Next)),

                        Accept::Return(value) => {
                            if let Some(value) = value {
                                break Some(Ok(Accept::Push(Capture::Value(
                                    value,
                                    None,
                                    self.severity,
                                ))));
                            } else {
                                break Some(Ok(Accept::Push(Capture::Empty)));
                            }
                        }

                        Accept::Repeat(value) => {
                            if let Some(value) = value {
                                results.push(value);
                            }
                        }

                        Accept::Push(mut capture) if capture.get_severity() > self.severity => {
                            capture.set_severity(self.severity);
                            break Some(Ok(Accept::Push(capture)));
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
                            && context
                                .runtime
                                .reader
                                .capture_from(&context.reader_start)
                                .len()
                                == 0
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
            } else if !first && context.runtime.reader.eof() {
                state = Some(false);
            } else {
                state = None;
            }

            // Reset capture stack for loop repeat
            context.runtime.stack.truncate(context.capture_start);
            context.reader_start = context.runtime.reader.tell();

            first = false;
        };

        match result {
            Some(result) if !matches!(result, Ok(Accept::Next)) => result,
            _ => {
                if results.len() > 1 {
                    Ok(Accept::Push(Capture::Value(
                        Value::List(Box::new(results)).into(),
                        None,
                        self.severity,
                    )))
                } else if results.len() == 1 {
                    Ok(Accept::Push(Capture::Value(
                        results.pop().unwrap(),
                        None,
                        self.severity,
                    )))
                } else {
                    Ok(Accept::Push(Capture::Empty))
                }
            }
        }
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
        depth: usize,
    ) -> Result<Accept, Reject> {
        // Check for a previously memoized result in memo table
        let id = self as *const Parselet as usize;

        // When parselet is consuming, try to read previous result from cache.
        if self.consuming.is_some() {
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
            &self,
            self.locals,
            args,
            if main { self.locals } else { 0 }, // Hold runtime globals when this is main!
            depth,
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
                            *var = Capture::Value(value.clone(), None, 0);
                            continue;
                        }
                    }

                    if let Some(addr) = arg.1 {
                        // fixme: This might leak the immutablestatic value to something mutable...
                        *var =
                            Capture::Value(context.runtime.program.statics[addr].clone(), None, 0);
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
                    Capture::Value(Value::Void.into(), None, 0);
            }
        }

        //println!("remaining {:?}", nargs);

        // Perform left-recursive execution
        let result = if let Some(true) = self.consuming {
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
                    .resize(context.capture_start, Capture::Empty);
            }

            context.runtime.reader.reset(reader_end);

            result
        } else {
            let result = self._run(&mut context, main);

            if !main && self.consuming.is_some() {
                context.runtime.memo.insert(
                    (context.reader_start.offset, id),
                    (context.runtime.reader.tell(), result.clone()),
                );
            }

            result
        };

        if context.runtime.debug > 1 {
            loop {
                if let Ok(Accept::Push(Capture::Value(ref value, ..))) = result {
                    let value = value.borrow();
                    if let Some(d) = value.get_dict() {
                        if d.get("emit").is_some() {
                            context.debug("=> AST");
                            ast::print(&value);
                            break;
                        }
                    }
                }

                context.debug(&format!("=> {:?}", result));
                break;
            }
        }

        result
    }
}

impl Object for Parselet {
    fn name(&self) -> &str {
        "parselet"
    }

    fn repr(&self) -> String {
        if let Some(name) = &self.name {
            format!("\"<{} {}>\"", self.name().to_string(), name)
        } else {
            format!(
                "\"<{} {}>\"",
                self.name().to_string(),
                self as *const Self as usize
            )
        }
    }

    fn is_callable(&self, with_arguments: bool) -> bool {
        // Either without arguments and signature is empty or all arguments have default values
        (!with_arguments && (self.signature.len() == 0 || self.signature.iter().all(|arg| arg.1.is_some())))
        // or with arguments and signature exists
            || (with_arguments && self.signature.len() > 0)
    }

    fn is_consuming(&self) -> bool {
        self.consuming.is_some()
    }

    fn call(
        &self,
        context: &mut Context,
        args: usize,
        nargs: Option<Dict>,
    ) -> Result<Accept, Reject> {
        self.run(context.runtime, args, nargs, false, context.depth + 1)
    }
}

impl std::cmp::PartialEq for Parselet {
    // It satisfies to just compare the parselet's memory address for equality
    fn eq(&self, other: &Self) -> bool {
        self as *const Self as usize == other as *const Self as usize
    }
}

impl std::hash::Hash for Parselet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self as *const Self as usize).hash(state);
    }
}

impl std::cmp::PartialOrd for Parselet {
    // It satisfies to just compare the parselet's memory address for equality
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let left = self as *const Self as usize;
        let right = other as *const Self as usize;

        left.partial_cmp(&right)
    }
}
