//! Intermediate representation of a parselet
use super::*;
use crate::reader::Offset;
use crate::value::Parselet;
use std::cell::RefCell;

#[derive(Debug)]
/// Intermediate parselet
pub struct ImlParselet {
    pub offset: Option<Offset>,                     // Offset of definition
    pub consuming: Option<Consumable>,              // Consumable state
    pub severity: u8,                               // Capture push severity
    pub name: Option<String>,                       // Parselet's name from source (for debugging)
    pub constants: Vec<(String, Option<ImlValue>)>, // Constant signature with default constants; generic parselet when set.
    pub signature: Vec<(String, Option<ImlValue>)>, // Argument signature with default arguments
    pub locals: usize, // Total number of local variables present (including arguments)
    pub begin: ImlOp,  // Begin-operations
    pub end: ImlOp,    // End-operations
    pub body: ImlOp,   // Operations
}

/** Representation of parselet in intermediate code. */
impl ImlParselet {
    pub fn id(&self) -> usize {
        self as *const ImlParselet as usize
    }
}

impl std::cmp::PartialEq for ImlParselet {
    // It satisfies to just compare the parselet's memory address for equality
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for ImlParselet {}

impl std::hash::Hash for ImlParselet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl std::cmp::PartialOrd for ImlParselet {
    // It satisfies to just compare the parselet's memory address for equality
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id().partial_cmp(&other.id())
    }
}
