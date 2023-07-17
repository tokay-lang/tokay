//! Runtime thread withing a VM program.
use super::*;
use crate::reader::{Offset, Reader};
use crate::value::RefValue;
use crate::{Error, Object};
use std::collections::HashMap;

/** Describes a runtime thread within a VM program.

A runtime thread is a Reader on an input stream, a memoization table related to the reader
and the stack the VM operates on.
*/
pub struct Thread<'program, 'reader> {
    pub program: &'program Program,  // the program this thread belongs to
    pub reader: &'reader mut Reader, // reader to read from
    pub memo: HashMap<(usize, usize), (Offset, Result<Accept, Reject>)>, // parselet memoization table
    pub stack: Vec<Capture>,                                             // VM value stack
    pub debug: u8,                                                       // Debug level
}

impl<'program, 'reader> Thread<'program, 'reader> {
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

    pub fn save_stack(&mut self) -> Vec<RefValue> {
        self.stack.drain(..).map(|item| item.get_value()).collect()
    }

    pub fn reset(&mut self) {
        self.memo.clear();
        self.stack.clear();
    }

    pub fn run(&mut self) -> Result<Option<RefValue>, Error> {
        match self
            .program
            .main()
            .0
            .borrow()
            .run(self, self.stack.len(), None, true, 0)
        {
            Ok(Accept::Push(Capture::Value(value, ..))) => {
                if value.is_void() {
                    Ok(None)
                } else {
                    Ok(Some(value.clone()))
                }
            }
            Ok(_) => Ok(None),
            Err(Reject::Error(error)) => Err(*error),
            Err(other) => Err(Error::new(None, format!("Runtime error {:?}", other))),
        }
    }
}
