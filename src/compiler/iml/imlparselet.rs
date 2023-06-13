//! Intermediate representation of a parselet
use super::*;
use crate::reader::Offset;
use indexmap::IndexMap;

#[derive(Debug)]
/// Intermediate parselet
pub(in crate::compiler) struct ImlParselet {
    pub offset: Option<Offset>,                // Offset of definition
    pub consuming: bool,                       // Flag if parselet is consuming
    pub severity: u8,                          // Capture push severity
    pub name: Option<String>,                  // Parselet's name from source (for debugging)
    pub constants: IndexMap<String, ImlValue>, // Parselet generic signature with default configuration
    pub signature: IndexMap<String, ImlValue>, // Argument signature with default arguments
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
