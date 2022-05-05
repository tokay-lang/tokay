//! Contexts represent stack frames for parselet calls.

use std::iter::FromIterator;

use super::*;
use crate::reader::Offset;
use crate::value::{Dict, List, Object, Parselet, RefValue};

/** Contexts represent stack frames for parselet calls.

Via the context, most operations regarding capture storing and loading is performed. */
pub struct Context<'runtime, 'program, 'reader, 'parselet> {
    pub(crate) runtime: &'runtime mut Runtime<'program, 'reader>, // Overall runtime
    pub(crate) parselet: &'parselet Parselet, // Current parselet that is executed
    pub(crate) stack_start: usize,            // Stack start (including locals and parameters)
    pub(crate) capture_start: usize,          // Stack capturing start
    pub(crate) reader_start: Offset,          // Current reader offset
    pub(crate) source_offset: Option<Offset>, // Tokay source offset
    hold: usize,             // Defines number of stack items to hold on context drop
    pub(crate) depth: usize, // Recursion depth
}

impl<'runtime, 'program, 'reader, 'parselet> Context<'runtime, 'program, 'reader, 'parselet> {
    pub fn new(
        runtime: &'runtime mut Runtime<'program, 'reader>,
        parselet: &'parselet Parselet,
        locals: usize,
        take: usize,
        hold: usize,
        depth: usize,
    ) -> Self {
        let stack_start = runtime.stack.len() - take;

        /*
        println!("--- {:?} ---", parselet.name);
        println!("stack = {:#?}", runtime.stack);
        println!("stack = {:?}", runtime.stack.len());
        println!("start = {:?}", stack_start);
        println!("resize = {:?}", stack_start + locals + 1);
        */

        runtime
            .stack
            .resize(stack_start + locals + 1, Capture::Empty);

        Self {
            stack_start,
            capture_start: stack_start + locals + 1,
            reader_start: runtime.reader.tell(),
            runtime,
            parselet,
            source_offset: None,
            hold,
            depth,
        }
    }

    /// Print debug output with context depth indention
    #[inline]
    pub fn debug(&self, msg: &str) {
        println!(
            "{}{}{:5} {}",
            ".".repeat(self.depth),
            //self.parselet.name.as_deref().unwrap_or("(unnamed)"), // fixme: TEMPORAL!
            self.parselet.name,
            //if self.parselet.consuming.is_some() {
            if self.parselet.consuming.is_some() {
                format!("@{: <4}", self.runtime.reader.tell().offset)
            } else {
                "".to_string()
            },
            msg
        );
    }

    /// Shortcut for an Ok(Accept::Push) with the given value.
    /// To push a value immediatelly, use context.runtime.stack.push().
    // fixme: Replace later by real push, when recursive VM is removed.
    #[inline]
    pub fn push(&self, value: RefValue) -> Result<Accept, Reject> {
        Ok(Accept::Push(Capture::Value(value, None, 10)))
    }

    /// Pop value off the stack.
    #[inline]
    pub fn pop(&mut self) -> RefValue {
        // todo: check for context limitations on the stack?
        let mut capture = self.runtime.stack.pop().unwrap();
        capture.extract(self.runtime.reader)
    }

    /// Peek top value of stack.
    #[inline]
    pub fn peek(&mut self) -> RefValue {
        // todo: check for context limitations on the stack?
        let capture = self.runtime.stack.last_mut().unwrap();
        capture.extract(self.runtime.reader)
    }

    // Push a value onto the stack
    #[inline]
    pub fn load(&mut self, index: usize) -> Result<Accept, Reject> {
        let capture = &mut self.runtime.stack[index];
        let value = capture.extract(self.runtime.reader);
        self.push(value)
    }

    /** Return a capture by index as RefValue. */
    pub fn get_capture(&mut self, pos: usize) -> Option<RefValue> {
        let pos = self.capture_start + pos - 1;

        if pos >= self.runtime.stack.len() {
            None
        }
        // This is $0?
        else if pos < self.capture_start {
            // Capture 0 either returns an already set value or ...
            if let Capture::Value(value, ..) = &self.runtime.stack[pos] {
                return Some(value.clone());
            }

            // ...returns the current range read so far.
            Some(RefValue::from(
                self.runtime
                    .reader
                    .get(&self.runtime.reader.capture_from(&self.reader_start)),
            ))
        // Any other index.
        } else {
            self.runtime.stack[pos].degrade();
            Some(self.runtime.stack[pos].extract(&self.runtime.reader))
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
                        return Some(capture.extract(self.runtime.reader));
                    }
                }
                _ => {}
            }
        }

        None
    }

    /** Set a capture to a RefValue by index. */
    pub fn set_capture(&mut self, pos: usize, value: RefValue) {
        let pos = self.capture_start + pos - 1;

        if pos >= self.runtime.stack.len() {
            return;
        }

        let capture = &mut self.runtime.stack[pos];

        // $0 gets a higher severity than normal captures.
        let severity = if pos < self.capture_start { 10 } else { 5 };

        match capture {
            Capture::Empty => *capture = Capture::Value(value, None, severity),
            Capture::Range(_, alias, _) => {
                *capture = Capture::Value(value, alias.clone(), severity)
            }
            Capture::Value(capture_value, ..) => {
                *capture_value = value;
            }
        }
    }

    /** Set a capture to a RefValue by name. */
    pub fn set_capture_by_name(&mut self, name: &str, value: RefValue) {
        let tos = self.runtime.stack.len();

        for i in (0..tos - self.capture_start).rev() {
            let capture = &mut self.runtime.stack[self.capture_start + i];

            match capture {
                Capture::Range(_, alias, ..) | Capture::Value(_, alias, ..) if alias.is_some() => {
                    if alias.as_ref().unwrap() == name {
                        match capture {
                            Capture::Empty => *capture = Capture::Value(value, None, 5),
                            Capture::Range(_, alias, _) => {
                                *capture = Capture::Value(value, alias.clone(), 5)
                            }
                            Capture::Value(capture_value, ..) => {
                                *capture_value = value;
                            }
                        }
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
        if capture_start > self.runtime.stack.len() {
            return Ok(None);
        }

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

        if self.runtime.debug > 5 {
            self.debug(&format!(
                "collect captures = {} single = {}, severity = {}",
                captures.len(),
                single,
                severity
            ));
            for i in 0..captures.len() {
                self.debug(&format!(" {}: {:?}", i, captures[i]));
            }
        }

        // No captures, then just stop!
        if captures.len() == 0 {
            return Ok(None);
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

                    let value = RefValue::from(self.runtime.reader.get(&range));

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

                    if !value.is_void() {
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

        if self.runtime.debug > 6 {
            println!("list = {:?}", list);
            println!("dict = {:?}", dict);
        }

        if dict.len() == 0 {
            if list.len() > 1 || (list.len() > 0 && !single) {
                Ok(Some(RefValue::from(list)))
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

            Ok(Some(RefValue::from(dict)))
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
            .map(|mut capture| capture.extract(self.runtime.reader))
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
