//! Intermediate representation of a parselet
use super::*;
use crate::reader::Offset;
use crate::value::Parselet;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::rc::Rc;

// ImlParseletModel
// ----------------------------------------------------------------------------

/// Intermediate parselet model
#[derive(Debug)]
pub(in crate::compiler) struct ImlParseletModel {
    pub consuming: bool,                       // Flag if parselet is consuming
    pub signature: IndexMap<String, ImlValue>, // Arguments signature with default values
    pub locals: usize, // Total number of local variables present (including arguments)
    pub begin: ImlOp,  // Begin intermediate operations
    pub end: ImlOp,    // End intermediate operations
    pub body: ImlOp,   // Body intermediate Operations
}

impl ImlParseletModel {
    pub fn id(&self) -> usize {
        self as *const ImlParseletModel as usize
    }
}

// ImlParselet
// ----------------------------------------------------------------------------

/// Intermediate parselet
#[allow(dead_code)]
#[derive(Debug)]
pub(in crate::compiler) struct ImlParselet {
    pub model: Rc<RefCell<ImlParseletModel>>, // Parselet base model
    pub constants: IndexMap<String, ImlValue>, // Generic signature with default configuration
    pub offset: Option<Offset>,               // Offset of definition
    pub name: Option<String>,                 // Assigned name from source (for debugging)
    pub severity: u8,                         // Capture push severity
}

/** Representation of parselet in intermediate code. */
impl ImlParselet {
    pub fn new(
        model: ImlParseletModel,
        constants: IndexMap<String, ImlValue>,
        offset: Option<Offset>,
        name: Option<String>,
        severity: u8,
    ) -> Self {
        Self {
            model: Rc::new(RefCell::new(model)),
            constants,
            offset,
            name,
            severity,
        }
    }

    pub fn derive(&self, constants: IndexMap<String, ImlValue>, offset: Option<Offset>) -> Self {
        Self {
            model: self.model.clone(),
            constants,
            offset,
            name: self.name.clone(),
            severity: self.severity,
        }
    }

    pub fn id(&self) -> usize {
        self as *const ImlParselet as usize
    }

    pub fn compile(&self, program: &mut ImlProgram) -> Parselet {
        let model = self.model.borrow();

        Parselet::new(
            Some(format!("{}", self)),
            None,
            self.severity,
            model
                .signature
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
            model.locals,
            model.begin.compile_to_vec(program, self),
            model.end.compile_to_vec(program, self),
            model.body.compile_to_vec(program, self),
        )
    }
}

impl std::fmt::Display for ImlParselet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.name.as_deref().unwrap_or("<anonymous parselet>")
        )?;

        if !self.constants.is_empty() {
            write!(f, "<")?;
            for (i, (name, value)) in self.constants.iter().enumerate() {
                if matches!(value, ImlValue::Void) {
                    write!(f, "{}{}", if i > 0 { ", " } else { "" }, name)?;
                } else {
                    write!(f, "{}{}:{}", if i > 0 { ", " } else { "" }, name, value)?;
                }
            }
            write!(f, ">")?;
        }

        Ok(())
    }
}

impl std::cmp::PartialEq for ImlParselet {
    // It satisfies to just compare the parselet's memory address for equality
    fn eq(&self, other: &Self) -> bool {
        self.model.borrow().id() == other.model.borrow().id() && self.constants == other.constants
    }
}

impl Eq for ImlParselet {}

impl std::hash::Hash for ImlParselet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let model = &*self.model.borrow();
        (model as *const ImlParseletModel as usize).hash(state);
        self.constants.iter().collect::<Vec<_>>().hash(state);
    }
}

impl std::cmp::PartialOrd for ImlParselet {
    // It satisfies to just compare the parselet's memory address for equality
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id().partial_cmp(&other.id())
    }
}

impl From<ImlParselet> for ImlValue {
    fn from(parselet: ImlParselet) -> Self {
        ImlValue::Parselet(ImlSharedParselet::new(parselet))
    }
}

// ImlSharedParselet
// ----------------------------------------------------------------------------

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
