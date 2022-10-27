//! Holds overall required information for VM execution.

use std::collections::HashMap;

use super::*;
use crate::reader::{Offset, Reader};
use crate::value::RefValue;

/** Merges a program and a reader into one container.

Holds additional runtime information, like the stack or memoization table.
*/
pub struct Runtime<'program, 'reader> {
    pub program: &'program Program,      // program to execute
    pub reader: &'reader mut Reader,     // reader to read from
    pub writer: Box<dyn std::io::Write>, // writer to write to
    pub start: usize,                    // absolute start offset in relation to reader

    pub memo: HashMap<(usize, usize), (Offset, Result<Accept, Reject>)>, // memoization table
    pub stack: Vec<Capture>,                                             // value stack

    pub debug: u8, // Debug level
}

impl<'program, 'reader> Runtime<'program, 'reader> {
    pub fn new(program: &'program Program, reader: &'reader mut Reader) -> Self {
        Self {
            program,
            reader,
            writer: Box::new(std::io::stdout()),
            start: 0,
            memo: HashMap::new(),
            stack: Vec::new(),
            debug: if let Ok(level) = std::env::var("TOKAY_DEBUG") {
                level.parse::<u8>().unwrap_or_default()
            } else {
                0
            },
        }
    }

    pub fn load_stack(&mut self, stack: Vec<RefValue>) {
        for item in stack {
            self.stack.push(Capture::Value(item, None, 0));
        }
    }

    pub fn save_stack(mut self) -> Vec<RefValue> {
        self.stack.drain(..).map(|item| item.get_value()).collect()
    }
}
