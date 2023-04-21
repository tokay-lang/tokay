//! Intermediate representation of a parselet
use super::*;
use crate::reader::Offset;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::rc::Rc;

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

impl std::fmt::Display for ImlParselet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.name.as_deref().unwrap_or("<anonymous parselet>")
        )
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

/// Shared ImlParselet
#[derive(Clone, Eq, PartialEq)]
pub(in crate::compiler) struct ImlSharedParselet(Rc<RefCell<ImlParselet>>);

impl ImlSharedParselet {
    pub fn new(parselet: ImlParselet) -> Self {
        Self(Rc::new(RefCell::new(parselet)))
    }
}

impl std::fmt::Debug for ImlSharedParselet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = self.0.try_borrow_mut().is_ok();

        if ret {
            self.0.borrow().fmt(f)
        } else {
            write!(f, "{}", self.0.borrow())
        }
    }
}

impl std::fmt::Display for ImlSharedParselet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.borrow())
    }
}

impl std::ops::Deref for ImlSharedParselet {
    type Target = Rc<RefCell<ImlParselet>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ImlSharedParselet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
