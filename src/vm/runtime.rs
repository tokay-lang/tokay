//! Runtime environment for a VM program execution.
use super::*;
use crate::reader::{Offset, Reader};
use crate::value::RefValue;
use std::collections::HashMap;

/** Describes a runtime context for a VM program.

A runtime context is a Reader on an input stream, a memoization table related to the reader
and the stack the VM operates on.
*/
pub struct Runtime {
    pub reader: Reader, // reader to read from
    pub memo: HashMap<(usize, usize), (Offset, Result<Accept, Reject>)>, // parselet memoization table
    pub stack: Vec<Capture>,                                             // VM value stack
    pub debug: u8,                                                       // Debug level
}

impl Runtime {
    pub fn new(reader: Reader) -> Self {
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

    /*
        TODO: Implement a drop function that releases the reader
        (and maybe also output and error) for further use.
    */
}
