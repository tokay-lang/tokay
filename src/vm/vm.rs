use std::collections::HashMap;
use std::iter::FromIterator;

use super::*;
use crate::error::Error;
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
    pub fn from_value(value: RefValue) -> Self {
        Capture::Value(value, 10)
    }

    // Degrades a capture to a severity to a capture with zero severity.
    // This is done when a capture is read.
    pub fn degrade(&mut self) {
        match self {
            Capture::Range(range, severity) if *severity <= 5 => {
                *self = Capture::Range(range.clone(), 0)
            }
            Capture::Value(value, severity) if *severity <= 5 => {
                *self = Capture::Value(value.clone(), 0)
            }
            Capture::Named(capture, _) => (*capture).degrade(),
            _ => {}
        }
    }

    // Turns a capture into a stand-alone value.
    pub fn as_value(&self, runtime: &Runtime) -> RefValue {
        match self {
            Capture::Empty => Value::Void.into_refvalue(),

            Capture::Range(range, _) => {
                Value::String(runtime.reader.extract(range)).into_refvalue()
            }

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
    Skip,
    Return,
    Main,
    Error(Box<Error>),
}

// --- Context -----------------------------------------------------------------

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

        // Initialize locals
        for i in 0..parselet.locals {
            if let Capture::Empty = runtime.stack[stack_start + i] {
                runtime.stack[stack_start + i] = Capture::Value(Value::Void.into_refvalue(), 10);
            }
        }

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

    /// Peek top value of stack.
    pub fn peek(&mut self) -> RefValue {
        // todo: check for context limitations on the stack?
        let capture = self.runtime.stack.last().unwrap();
        capture.as_value(self.runtime)
    }

    /** Return a capture by index as RefValue. */
    pub fn get_capture(&mut self, pos: usize) -> Option<RefValue> {
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
                .into_refvalue(),
            )
        } else {
            self.runtime.stack[self.capture_start + pos].degrade();
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

        self.runtime.stack[pos] = Capture::Value(value, 10)
    }

    /** Set a capture to a RefValue by name. */
    pub fn set_capture_by_name(&mut self, name: &str, value: RefValue) {
        // fixme: Should be examined in reversed order
        for capture in self.runtime.stack[self.capture_start..].iter_mut() {
            if let Capture::Named(capture, alias) = capture {
                if alias == name {
                    *capture = Box::new(Capture::Value(value, 10));
                    break;
                }
            }
        }
    }

    /** Get slice of all captures from current context */
    pub fn get_captures(&self) -> &[Capture] {
        &self.runtime.stack[self.capture_start..]
    }

    //temporary...
    /** Drain all captures from current context */
    pub fn drain_captures(&mut self) -> Vec<Capture> {
        self.runtime.stack.drain(self.capture_start..).collect()
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
        min_severity: u8,
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

        if self.runtime.debug {
            println!("--- Collect ---");
            println!("single = {}, min_severity = {}", single, min_severity);
            println!("captures = {:?}", captures);
        }

        if captures.len() == 0 {
            None
        } else if single && captures.len() == 1 && !matches!(captures[0], Capture::Named(_, _)) {
            Some(captures.pop().unwrap())
        } else {
            let mut list = List::new();
            let mut dict = Dict::new();
            let mut max = min_severity;

            // Collect any significant captures and values
            for capture in captures.into_iter() {
                match capture {
                    Capture::Range(range, severity) if severity >= max => {
                        if severity > max {
                            max = severity;
                            list.clear();
                        }

                        list.push(
                            Value::String(self.runtime.reader.extract(&range)).into_refvalue(),
                        );
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

            if self.runtime.debug {
                println!("list = {:?}", list);
                println!("dict = {:?}", dict);
            }

            if dict.len() == 0 {
                if list.len() > 1 {
                    return Some(Capture::Value(
                        Value::List(Box::new(list)).into_refvalue(),
                        5,
                    ));
                } else if list.len() == 1 {
                    return Some(Capture::Value(list[0].clone(), 5));
                }

                None
            } else {
                /*
                // Store list-items additionally when there is a dict? Hmm...
                for (i, item) in list.into_iter().enumerate() {
                    dict.insert(i.to_string(), item);
                }
                */

                Some(Capture::Value(
                    Value::Dict(Box::new(dict)).into_refvalue(),
                    5,
                ))
            }
        }
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
            self.stack.push(Capture::Value(item, 10));
        }
    }

    pub fn save_stack(mut self) -> Vec<RefValue> {
        let mut ret = Vec::new();
        let stack: Vec<Capture> = self.stack.drain(..).collect();

        for item in stack {
            ret.push(item.as_value(&self));
        }

        ret
    }

    pub fn dump(&self) {
        println!("memo has {} entries", self.memo.len());
        println!("stack has {} entries", self.stack.len());
    }
}
