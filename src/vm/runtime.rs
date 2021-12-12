//! Holds overall required information for VM execution.

use std::collections::HashMap;

use super::*;
use crate::reader::{Offset, Reader};
use crate::value::RefValue;

/** Merges a program and a reader into one container.

Holds additional runtime information, like the stack or memoization table.
*/
pub struct Runtime<'program, 'reader> {
    pub(crate) program: &'program Program,
    pub(crate) reader: &'reader mut Reader,

    pub(crate) memo: HashMap<(usize, usize), (Offset, Result<Accept, Reject>)>,
    pub(crate) stack: Vec<Capture>,
    pub debug: u8, // Debug level
}

impl<'program, 'reader> Runtime<'program, 'reader> {
    pub fn new(program: &'program Program, reader: &'reader mut Reader) -> Self {
        Self {
            program,
            reader,
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

    pub fn dump(&self) {
        println!("memo has {} entries", self.memo.len());
        println!("stack has {} entries", self.stack.len());
    }
}
