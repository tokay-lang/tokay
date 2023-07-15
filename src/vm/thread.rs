//! Runtime thread withing a VM program.
use super::*;
use crate::reader::{Offset, Reader};
use crate::value::RefValue;
use std::collections::HashMap;

/** Describes a runtime thread within a VM program.

A runtime thread is a Reader on an input stream, a memoization table related to the reader
and the stack the VM operates on.
*/
pub struct Thread<'reader> {
    pub reader: &'reader mut Reader, // reader to read from
    pub memo: HashMap<(usize, usize), (Offset, Result<Accept, Reject>)>, // parselet memoization table
    pub stack: Vec<Capture>,                                             // VM value stack
    pub debug: u8,                                                       // Debug level
}

impl<'reader> Thread<'reader> {
    pub fn new(reader: &'reader mut Reader) -> Self {
        Self {
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

    pub fn save_stack(&mut self) -> Vec<RefValue> {
        self.stack.drain(..).map(|item| item.get_value()).collect()
    }

    pub fn reset(&mut self) {
        self.memo.clear();
        self.stack.clear();
    }
}
