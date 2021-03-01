use std::collections::HashMap;
use std::iter::FromIterator;

use super::*;
use crate::reader::{Offset, Range, Reader};
use crate::value::{Dict, List, RefValue, Value};

// --- Capture -----------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Capture {
    Empty,                       // Empty capture
    Range(Range, u8),            // Captured range from the input & severity
    Value(RefValue, u8),         // Captured value & severity
    Named(Box<Capture>, String), // Named
}

impl Capture {
    pub fn as_value(&self, runtime: &Runtime) -> RefValue {
        match self {
            Capture::Empty => Value::Void.into_ref(),

            Capture::Range(range, _) => Value::String(runtime.reader.extract(range)).into_ref(),

            Capture::Value(value, _) => value.clone(),

            Capture::Named(capture, _) => capture.as_value(runtime),
        }
    }
}

// --- Accept ------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Accept {
    Next,
    Skip,
    Push(Capture),
    Repeat(Option<RefValue>),
    Return(Option<RefValue>),
}

// --- Reject ------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Reject {
    Next,
    Return,
    Main,
    Error(String),
}

// --- Context -----------------------------------------------------------------

pub struct Context<'runtime, 'program, 'reader> {
    pub runtime: &'runtime mut Runtime<'program, 'reader>, // fixme: Temporary pub?

    pub(super) stack_start: usize,
    pub(super) capture_start: usize,
    pub(super) reader_start: Offset,
}

impl<'runtime, 'program, 'reader> Context<'runtime, 'program, 'reader> {
    pub fn new(runtime: &'runtime mut Runtime<'program, 'reader>, preserve: usize) -> Self {
        let stack_start = runtime.stack.len();

        runtime
            .stack
            .resize(stack_start + preserve + 1, Capture::Empty);

        Self {
            stack_start,
            capture_start: stack_start + preserve,
            reader_start: runtime.reader.tell(),
            runtime: runtime,
        }
    }

    // Push value onto the stack
    pub fn push(&mut self, value: RefValue) {
        self.runtime.stack.push(Capture::Value(value, 10))
    }

    /// Pop value off the stack.
    pub fn pop(&mut self) -> RefValue {
        // todo: check for context limitations on the stack?
        let capture = self.runtime.stack.pop().unwrap();
        capture.as_value(self.runtime)
    }

    /** Return a capture by index as RefValue. */
    pub fn get_capture(&self, pos: usize) -> Option<RefValue> {
        if self.capture_start + pos >= self.runtime.stack.len() {
            return None;
        }

        if pos == 0 {
            // Capture 0 either returns an already set value or ...
            if let Capture::Value(value, _) = &self.runtime.stack[self.capture_start] {
                return Some(value.clone());
            }

            // ...returns the current range read so far.
            Some(
                Value::String(
                    self.runtime
                        .reader
                        .extract(&self.runtime.reader.capture_from(&self.reader_start)),
                )
                .into_ref(),
            )
        } else {
            Some(self.runtime.stack[self.capture_start + pos].as_value(&self.runtime))
        }
    }

    /** Return a capture by name as RefValue. */
    pub fn get_capture_by_name(&self, name: &str) -> Option<RefValue> {
        // fixme: Should be examined in reversed order
        for capture in self.runtime.stack[self.capture_start..].iter() {
            if let Capture::Named(capture, alias) = &capture {
                if alias == name {
                    return Some(capture.as_value(&self.runtime));
                }
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

        self.runtime.stack[pos] = Capture::Value(value, 5)
    }

    /** Set a capture to a RefValue by name. */
    pub fn set_capture_by_name(&mut self, name: &str, value: RefValue) {
        // fixme: Should be examined in reversed order
        for capture in self.runtime.stack[self.capture_start..].iter_mut() {
            if let Capture::Named(capture, alias) = capture {
                if alias == name {
                    *capture = Box::new(Capture::Value(value, 5));
                    break;
                }
            }
        }
    }

    /** Get slice of all captures from current context */
    pub fn get_captures(&self) -> &[Capture] {
        &self.runtime.stack[self.capture_start..]
    }

    /** Drain all captures from current context */
    pub fn drain_captures(&mut self) -> Vec<Capture> {
        self.runtime.stack.drain(self.capture_start..).collect()
    }

    /** Helper function to collect captures from a capture_start and turn
    them either into a dict or list object capture or take them as is.

    This function is internally used for automatic AST construction and value
    inheriting.
    */
    pub(super) fn collect(
        &mut self,
        capture_start: usize,
        copy: bool,
        single: bool,
    ) -> Option<Capture> {
        // Eiter copy or drain captures from stack
        let mut captures: Vec<Capture> = if copy {
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

        //println!("captures = {:?}", captures);

        if captures.len() == 0 {
            None
        } else if single && captures.len() == 1 && !matches!(captures[0], Capture::Named(_, _)) {
            Some(captures.pop().unwrap())
        } else {
            let mut list = List::new();
            let mut dict = Dict::new();
            let mut max = 0;

            // Collect any significant captures and values
            for capture in captures.into_iter() {
                match capture {
                    Capture::Range(range, severity) if severity >= max => {
                        if severity > max {
                            max = severity;
                            list.clear();
                        }

                        list.push(Value::String(self.runtime.reader.extract(&range)).into_ref());
                    }

                    Capture::Value(value, severity) if severity >= max => {
                        if severity > max {
                            max = severity;
                            list.clear();
                        }

                        list.push(value);
                    }

                    Capture::Named(capture, alias) => {
                        // Named capture becomes dict key
                        dict.insert(alias, capture.as_value(self.runtime));
                    }

                    _ => continue,
                };
            }

            //println!("list = {:?}", list);
            //println!("dict = {:?}", dict);

            if dict.len() == 0 {
                if list.len() > 1 {
                    return Some(Capture::Value(Value::List(Box::new(list)).into_ref(), 5));
                } else if list.len() == 1 {
                    return Some(Capture::Value(list[0].clone(), 5));
                }

                None
            } else {
                for (i, item) in list.into_iter().enumerate() {
                    dict.insert(i.to_string(), item);
                }

                if dict.len() == 1 {
                    return Some(Capture::Value(dict.values().next().unwrap().clone(), 5));
                }

                Some(Capture::Value(Value::Dict(Box::new(dict)).into_ref(), 5))
            }
        }
    }
}

impl<'runtime, 'program, 'reader> Drop for Context<'runtime, 'program, 'reader> {
    fn drop(&mut self) {
        self.runtime.stack.truncate(self.stack_start);
    }
}

// --- Runtime -----------------------------------------------------------------

pub struct Runtime<'program, 'reader> {
    pub(super) program: &'program Program,
    pub(crate) reader: &'reader mut Reader, // temporary pub

    pub(super) memo: HashMap<(usize, usize), (Offset, Result<Accept, Reject>)>,
    pub(super) stack: Vec<Capture>,
}

impl<'program, 'reader> Runtime<'program, 'reader> {
    pub fn new(program: &'program Program, reader: &'reader mut Reader) -> Self {
        Self {
            program,
            reader,
            memo: HashMap::new(),
            stack: Vec::new(),
        }
    }

    pub fn dump(&self) {
        println!("memo has {} entries", self.memo.len());
        println!("stack has {} entries", self.stack.len());
    }
}
