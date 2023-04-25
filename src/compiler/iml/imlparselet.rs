//! Intermediate representation of a parselet
use super::*;
use crate::reader::Offset;
use crate::value::Parselet;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::rc::Rc;

/// Intermediate parselet
#[derive(Debug)]
pub(in crate::compiler) struct ImlParselet {
    pub offset: Option<Offset>,                // Offset of definition
    pub consuming: bool,                       // Flag if parselet is consuming
    pub severity: u8,                          // Capture push severity
    pub name: Option<String>,                  // Assigned name from source (for debugging)
    pub signature: IndexMap<String, ImlValue>, // Arguments signature with default values
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

    pub fn compile(&self, program: &mut ImlProgram) -> Parselet {
        Parselet::new(
            self.name.clone(),
            None,
            self.severity,
            self.signature
                .iter()
                .map(|var_value| {
                    (
                        // Copy parameter name
                        var_value.0.clone(),
                        // Register default value, if any
                        match &var_value.1 {
                            ImlValue::Void => None,
                            value => Some(program.register(value).expect("Cannot register value")),
                        },
                    )
                })
                .collect(),
            self.locals,
            self.begin.compile_to_vec(program),
            self.end.compile_to_vec(program),
            self.body.compile_to_vec(program),
        )
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
        // Avoid endless recursion in case of recursive parselets
        if self.0.try_borrow_mut().is_ok() {
            self.0.borrow().fmt(f)
        } else {
            write!(f, "{} (recursive)", self.0.borrow())
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
