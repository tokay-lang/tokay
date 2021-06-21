use std::collections::HashMap;
use std::iter::FromIterator;

use super::*;
use crate::error::Error;
use crate::reader::{Offset, Range, Reader};
use crate::value::{Dict, List, RefValue, Value};

// --- Capture -----------------------------------------------------------------

/// Captures are stack items where the VM operates on.
#[derive(Debug, Clone)]
pub enum Capture {
    Empty,                               // Empty capture
    Range(Range, Option<String>, u8),    // Captured range
    Value(RefValue, Option<String>, u8), // Captured value
}

impl Capture {
    fn from_value(&mut self, from: RefValue) {
        match self {
            Capture::Empty => *self = Capture::Value(from, None, 5),
            Capture::Range(_, alias, _) => *self = Capture::Value(from, alias.clone(), 5),
            Capture::Value(value, ..) => {
                *value = from;
            }
        }
    }

    // Turns a capture into a value.
    fn into_value(&mut self, reader: &Reader) -> RefValue {
        match self {
            Capture::Empty => Value::Void.into_refvalue(),
            Capture::Range(range, alias, severity) => {
                let value = Value::String(reader.extract(range)).into_refvalue();
                *self = Capture::Value(value.clone(), alias.clone(), *severity);
                value
            }
            Capture::Value(value, ..) => value.clone(),
        }
    }

    pub fn get_value(&self) -> RefValue {
        match self {
            Capture::Empty => Value::Void.into_refvalue(),
            Capture::Range(..) => {
                panic!("Cannot retrieve value of Capture::Range, use .into_value() first!")
            }
            Capture::Value(value, ..) => value.clone(),
        }
    }

    // Degrades a capture to a severity to a capture with zero severity.
    // This is done when a capture is read.
    pub fn degrade(&mut self) {
        match self {
            Capture::Range(_, _, severity) | Capture::Value(_, _, severity) if *severity <= 5 => {
                *severity = 0;
            }
            _ => {}
        }
    }
}

// --- Accept ------------------------------------------------------------------

/// Representing the Ok-value result on a branched run of the VM.
#[derive(Debug, Clone)]
pub enum Accept {
    Next,
    Skip,
    Push(Capture),
    Repeat(Option<RefValue>),
    Return(Option<RefValue>),
}

// --- Reject ------------------------------------------------------------------

/// Representing the Err-value result on a branched run of the VM.
#[derive(Debug, Clone)]
pub enum Reject {
    Next,
    Skip,
    Return,
    Main,
    Error(Box<Error>),
}

impl From<Error> for Reject {
    fn from(error: Error) -> Self {
        Reject::Error(Box::new(error))
    }
}

// --- Context -----------------------------------------------------------------

/** Contexts represent stack frames for function calls.

Via the context, most operations regarding capture storing and loading is performed. */
pub struct Context<'runtime, 'program, 'reader, 'parselet> {
    pub(crate) runtime: &'runtime mut Runtime<'program, 'reader>, // Overall runtime
    pub(crate) parselet: &'parselet Parselet, // Current parselet that is executed
    pub(crate) stack_start: usize,            // Stack start (including locals and parameters)
    pub(crate) capture_start: usize,          // Stack capturing start
    pub(crate) reader_start: Offset,          // Current reader offset
    pub(super) source_offset: Option<Offset>, // Tokay source offset
    hold: usize, // Defines number of stack items to hold on context drop
}

impl<'runtime, 'program, 'reader, 'parselet> Context<'runtime, 'program, 'reader, 'parselet> {
    pub fn new(
        runtime: &'runtime mut Runtime<'program, 'reader>,
        parselet: &'parselet Parselet,
        take: usize,
        hold: usize,
    ) -> Self {
        let stack_start = runtime.stack.len() - take;

        /*
        println!("---");
        println!("stack = {:#?}", runtime.stack);
        println!("stack = {:?}", runtime.stack.len());
        println!("start = {:?}", stack_start);
        println!("resize = {:?}", stack_start + preserve + 1);
        */

        runtime
            .stack
            .resize(stack_start + parselet.locals + 1, Capture::Empty);

        Self {
            stack_start,
            capture_start: stack_start + parselet.locals,
            reader_start: runtime.reader.tell(),
            runtime,
            parselet,
            source_offset: None,
            hold,
        }
    }

    /// Shortcut for an Ok(Accept::Push) with the given value.
    /// To push a value immediatelly, use context.runtime.stack.push().
    pub fn push(&self, value: RefValue) -> Result<Accept, Reject> {
        Ok(Accept::Push(Capture::Value(value, None, 10)))
    }

    /// Pop value off the stack.
    pub fn pop(&mut self) -> RefValue {
        // todo: check for context limitations on the stack?
        let mut capture = self.runtime.stack.pop().unwrap();
        capture.into_value(self.runtime.reader)
    }

    /// Peek top value of stack.
    pub fn peek(&mut self) -> RefValue {
        // todo: check for context limitations on the stack?
        let capture = self.runtime.stack.last_mut().unwrap();
        capture.into_value(self.runtime.reader)
    }

    // Push a value onto the stack
    pub fn load(&mut self, index: usize) -> Result<Accept, Reject> {
        let capture = &mut self.runtime.stack[index];
        let value = capture.into_value(self.runtime.reader);
        self.push(value)
    }

    /** Return a capture by index as RefValue. */
    pub fn get_capture(&mut self, pos: usize) -> Option<RefValue> {
        if self.capture_start + pos >= self.runtime.stack.len() {
            return None;
        }

        if pos == 0 {
            // Capture 0 either returns an already set value or ...
            if let Capture::Value(value, ..) = &self.runtime.stack[self.capture_start] {
                return Some(value.clone());
            }

            // ...returns the current range read so far.
            Some(
                Value::String(
                    self.runtime
                        .reader
                        .extract(&self.runtime.reader.capture_from(&self.reader_start)),
                )
                .into_refvalue(),
            )
        } else {
            self.runtime.stack[self.capture_start + pos].degrade();
            Some(self.runtime.stack[self.capture_start + pos].into_value(&self.runtime.reader))
        }
    }

    /** Return a capture by name as RefValue. */
    pub fn get_capture_by_name(&mut self, name: &str) -> Option<RefValue> {
        let tos = self.runtime.stack.len();

        for i in (0..tos - self.capture_start).rev() {
            let capture = &mut self.runtime.stack[self.capture_start + i];

            match capture {
                Capture::Range(_, alias, ..) | Capture::Value(_, alias, ..) if alias.is_some() => {
                    if alias.as_ref().unwrap() == name {
                        capture.degrade();
                        return Some(capture.into_value(self.runtime.reader));
                    }
                }
                _ => {}
            }
        }

        None
    }

    /** Set a capture to a RefValue by index. */
    pub fn set_capture(&mut self, pos: usize, value: RefValue) {
        let pos = self.capture_start + pos;

        if pos >= self.runtime.stack.len() {
            return;
        }

        self.runtime.stack[pos].from_value(value);
    }

    /** Set a capture to a RefValue by name. */
    pub fn set_capture_by_name(&mut self, name: &str, value: RefValue) {
        let tos = self.runtime.stack.len();

        for i in (0..tos - self.capture_start).rev() {
            let capture = &mut self.runtime.stack[self.capture_start + i];

            match capture {
                Capture::Range(_, alias, ..) | Capture::Value(_, alias, ..) if alias.is_some() => {
                    if alias.as_ref().unwrap() == name {
                        capture.from_value(value);
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    /** Helper function to collect captures from a capture_start and turn
    them either into a dict or list object capture or take them as is.

    This function is internally used for automatic AST construction and value
    inheriting.
    */
    pub(crate) fn collect(
        &mut self,
        capture_start: usize,
        copy: bool,
        single: bool,
        mut inherit: bool,
        severity: u8,
    ) -> Result<Option<RefValue>, Capture> {
        // Eiter copy or drain captures from stack
        let captures: Vec<Capture> = if copy {
            Vec::from_iter(
                self.runtime.stack[capture_start..]
                    .iter()
                    .filter(|item| !(matches!(item, Capture::Empty)))
                    .cloned(),
            )
        } else {
            self.runtime
                .stack
                .drain(capture_start..)
                .filter(|item| !(matches!(item, Capture::Empty)))
                .collect()
        };

        if self.runtime.debug {
            println!("--- Collect ---");
            println!("single = {}, severity = {}", single, severity);
            println!("captures = {:?}", captures);
        }

        // No captures, then just stop!
        if captures.len() == 0 {
            return Ok(None)
        }

        let mut list = List::new();
        let mut dict = Dict::new();
        let mut max = severity;

        // Capture inheritance is only possible when there is only one capture
        if inherit && captures.len() > 1 {
            inherit = false;
        }

        // Collect any significant captures and values
        for capture in captures.into_iter() {
            match capture {
                Capture::Range(range, alias, severity) if severity >= max => {
                    if severity > max {
                        max = severity;
                        list.clear();
                        dict.clear();
                    }

                    let value = Value::String(self.runtime.reader.extract(&range)).into_refvalue();

                    if let Some(alias) = alias {
                        dict.insert(alias, value);
                    } else if inherit {
                        return Err(Capture::Range(range, alias, severity));
                    } else {
                        list.push(value);
                    }
                }

                Capture::Value(value, alias, severity) if severity >= max => {
                    if severity > max {
                        max = severity;
                        list.clear();
                        dict.clear();
                    }

                    if !value.borrow().is_void() {
                        if let Some(alias) = alias {
                            dict.insert(alias, value);
                        } else if inherit {
                            return Err(Capture::Value(value, alias, severity));
                        } else {
                            list.push(value);
                        }
                    }
                }

                _ => {}
            };
        }

        if self.runtime.debug {
            println!("list = {:?}", list);
            println!("dict = {:?}", dict);
        }

        if dict.len() == 0 {
            if list.len() > 1 || (list.len() > 0 && !single) {
                Ok(Some(Value::List(Box::new(list)).into_refvalue()))
            } else if list.len() == 1 {
                Ok(Some(list.pop().unwrap()))
            } else {
                Ok(None)
            }
        } else {
            // Store list-items additionally when there is a dict?
            // This is currently under further consideration and not finished.
            let mut idx = 0;
            for item in list.into_iter() {
                loop {
                    let key = format!("#{}", idx);
                    if let None = dict.get(&key) {
                        dict.insert(key, item);
                        break;
                    }

                    idx += 1;
                }

                idx += 1;
            }

            Ok(Some(Value::Dict(Box::new(dict)).into_refvalue()))
        }
    }

    /// Drains n items off the stack into a vector of values
    pub(crate) fn drain(&mut self, n: usize) -> Vec<RefValue> {
        let tos = self.runtime.stack.len();
        assert!(n <= tos - self.capture_start);

        let captures: Vec<Capture> = self
            .runtime
            .stack
            .drain(tos - n..)
            .filter(|capture| !matches!(capture, Capture::Empty))
            .collect();

        captures
            .into_iter()
            .map(|mut capture| capture.into_value(self.runtime.reader))
            .collect()
    }
}

impl<'runtime, 'program, 'reader, 'parselet> Drop
    for Context<'runtime, 'program, 'reader, 'parselet>
{
    fn drop(&mut self) {
        self.runtime.stack.truncate(self.stack_start + self.hold);
    }
}

// --- Runtime -----------------------------------------------------------------

/** Merges a program and a reader into one container.

Holds additional runtime information, like the stack or memoization table */
pub struct Runtime<'program, 'reader> {
    pub(crate) program: &'program Program,
    pub(crate) reader: &'reader mut Reader,

    pub(super) memo: HashMap<(usize, usize), (Offset, Result<Accept, Reject>)>,
    pub(crate) stack: Vec<Capture>,
    pub debug: bool,
}

impl<'program, 'reader> Runtime<'program, 'reader> {
    pub fn new(program: &'program Program, reader: &'reader mut Reader) -> Self {
        Self {
            program,
            reader,
            memo: HashMap::new(),
            stack: Vec::new(),
            debug: false,
        }
    }

    pub fn load_stack(&mut self, stack: Vec<RefValue>) {
        for item in stack {
            self.stack.push(Capture::Value(item, None, 0));
        }
    }

    pub fn save_stack(mut self) -> Vec<RefValue> {
        let mut ret = Vec::new();
        let stack: Vec<Capture> = self.stack.drain(..).collect();

        for item in stack {
            ret.push(item.get_value());
        }

        ret
    }

    pub fn dump(&self) {
        println!("memo has {} entries", self.memo.len());
        println!("stack has {} entries", self.stack.len());
    }
}
