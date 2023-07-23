//! Runtime thread withing a VM program.
use super::*;
use crate::reader::{Offset, Reader};
use crate::value::RefValue;
use crate::{Error, Object};
use std::collections::HashMap;

/** Thread which is executing a VM program.

Holds runtime-specific information like the stack, readers and the packrat memoization table.
*/
pub struct Thread<'program, 'reader> {
    pub program: &'program Program, // the program this thread belongs to

    pub reader: &'reader mut Reader,       // Current reader
    pub readers: Vec<&'reader mut Reader>, // List of readers

    pub memo: HashMap<(usize, usize), (Offset, Result<Accept, Reject>)>, // parselet memoization table
    pub globals: Vec<RefValue>,                                          // Global variables
    pub debug: u8,                                                       // Debug level
}

impl<'program, 'reader> Thread<'program, 'reader> {
    pub fn new(program: &'program Program, mut readers: Vec<&'reader mut Reader>) -> Self {
        assert!(readers.len() > 0, "Expecting at least one reader");

        Self {
            program,
            reader: readers.remove(0), // first reader becomes current reader
            readers,                   // other readers are kept for later use
            memo: HashMap::new(),
            globals: Vec::new(),
            debug: if let Ok(level) = std::env::var("TOKAY_DEBUG") {
                level.parse::<u8>().unwrap_or_default()
            } else {
                0
            },
        }
    }

    pub fn run(&mut self) -> Result<Option<RefValue>, Error> {
        match self
            .program
            .main()
            .0
            .borrow()
            .run(self, Vec::new(), None, true, 0)
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
