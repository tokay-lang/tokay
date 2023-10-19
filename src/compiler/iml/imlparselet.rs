//! Intermediate representation of a parselet
use super::*;
use crate::reader::Offset;
use crate::value::Parselet;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::rc::Rc;

// ImlParseletModel
// ----------------------------------------------------------------------------

/** Intermediate parselet model.

The model defines the code and local varibles of the parselet, and is shared by
several parselet configurations. */
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

// ImlParseletConfig
// ----------------------------------------------------------------------------

/** Intermediate parselet configuration.

A parselet configuration is a model with as given constants definition.
The constants definition might be generic, which needs to be resolved first
before a parselet configuration is turned into a parselet.
*/
#[allow(dead_code)]
#[derive(Debug)]
pub(in crate::compiler) struct ImlParseletConfig {
    pub model: Rc<RefCell<ImlParseletModel>>, // Parselet base model
    pub constants: IndexMap<String, ImlValue>, // Generic signature with default configuration
    pub offset: Option<Offset>,               // Offset of definition
    pub name: Option<String>,                 // Assigned name from source (for debugging)
    pub severity: u8,                         // Capture push severity
    pub generated: bool,
}

/** Representation of parselet in intermediate code. */
impl ImlParseletConfig {
    pub fn new(
        model: ImlParseletModel,
        constants: IndexMap<String, ImlValue>,
        offset: Option<Offset>,
        name: Option<String>,
        severity: u8,
        generated: bool,
    ) -> Self {
        Self {
            model: Rc::new(RefCell::new(model)),
            constants,
            offset,
            name,
            severity,
            generated,
        }
    }

    pub fn id(&self) -> usize {
        self as *const ImlParseletConfig as usize
    }
}

impl std::fmt::Display for ImlParseletConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.name.as_deref().unwrap_or("<anonymous parselet>")
        )?;

        if !self.constants.is_empty() {
            write!(f, "<")?;
            for (i, (name, value)) in self.constants.iter().enumerate() {
                if matches!(value, ImlValue::Unset) {
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

impl std::cmp::PartialEq for ImlParseletConfig {
    // It satisfies to just compare the parselet's memory address for equality
    fn eq(&self, other: &Self) -> bool {
        self.model.borrow().id() == other.model.borrow().id() && self.constants == other.constants
    }
}

impl Eq for ImlParseletConfig {}

impl std::hash::Hash for ImlParseletConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let model = &*self.model.borrow();
        (model as *const ImlParseletModel as usize).hash(state);
        self.constants.iter().collect::<Vec<_>>().hash(state);
    }
}

impl std::cmp::PartialOrd for ImlParseletConfig {
    // It satisfies to just compare the parselet's memory address for equality
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id().partial_cmp(&other.id())
    }
}

impl From<ImlParseletConfig> for ImlValue {
    fn from(parselet: ImlParseletConfig) -> Self {
        ImlValue::Parselet(ImlParselet::new(parselet))
    }
}

// ImlParselet
// ----------------------------------------------------------------------------

/// Shared ImlParseletConfig
#[derive(Clone, Eq, PartialEq)]
pub(in crate::compiler) struct ImlParselet(Rc<RefCell<ImlParseletConfig>>);

impl ImlParselet {
    pub fn new(parselet: ImlParseletConfig) -> Self {
        Self(Rc::new(RefCell::new(parselet)))
    }

    /// Checks if an intermediate parselet is completely resolved, or if it has open generics
    pub fn is_generic(&self) -> bool {
        self.borrow()
            .constants
            .values()
            .any(|value| matches!(value, ImlValue::Generic { .. } | ImlValue::This(_)))
    }

    /** Derives an intermediate parselet by another intermediate parselet (`from`).

    The namespace defines the constant configuration of a surrounding parselet (`from`),
    and extends the intermediate parselet's constant configuration, making it a derivation.

    ```tokay
    A<X>: X           # intermediate generic parselet A
    B<Y>: 'x' A<Y>    # intermediate generic parselet B using a parselet instance of A

    B<'m'> B<'n'>     # parselet instances, construct the final parselets: B<'m'>, A<'m'>, B<'n'> A<'n'>
    ```

    The function either returns a derived parselet in case it was derive,
    otherwise it returns a clone of self.
    */
    pub fn derive(&self, from: &ImlParselet) -> Self {
        let mut constants = self.borrow().constants.clone();
        let mut changes = false;

        for value in constants.values_mut() {
            // Replace any generics until no more are open
            while let ImlValue::Generic { name, .. } = value {
                *value = from.borrow().constants.get(name).unwrap().clone();
                changes = true;
            }

            // Replace any values of self
            if let ImlValue::This(_) = value {
                *value = ImlValue::Parselet(from.clone());
                changes = true;
            }
        }

        // When there is no change, there is no derivation
        if !changes {
            return self.clone();
        }

        // Create derivation of the inner parselet
        let parselet = self.borrow();

        Self::new(ImlParseletConfig {
            model: parselet.model.clone(),
            constants,
            offset: parselet.offset.clone(),
            name: parselet.name.clone(),
            severity: parselet.severity,
            generated: parselet.generated,
        })
    }

    /** Compiles an intermediate parselet into a compiled VM parselet,
    which is part of the provided `program` and indexed by `this`. */
    pub fn compile(&self, program: &mut ImlProgram, this: usize) -> Parselet {
        let parselet = self.borrow();
        let model = parselet.model.borrow();

        Parselet::new(
            Some(format!("{}", parselet)),
            None,
            parselet.severity,
            model
                .signature
                .iter()
                .map(|var_value| {
                    (
                        // Copy parameter name
                        var_value.0.clone(),
                        // Register default value, if any
                        match &var_value.1 {
                            ImlValue::Unset => None,
                            value => Some(program.register(value)),
                        },
                    )
                })
                .collect(),
            model.locals,
            model.begin.compile_to_vec(program, (self, this)),
            model.end.compile_to_vec(program, (self, this)),
            model.body.compile_to_vec(program, (self, this)),
        )
    }
}

impl std::hash::Hash for ImlParselet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.borrow().hash(state);
    }
}

impl std::fmt::Debug for ImlParselet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.borrow())
        // Avoid endless recursion in case of recursive parselets
        /*
        if self.0.try_borrow_mut().is_ok() {
            self.0.borrow().fmt(f)
        } else {
            write!(f, "{} (recursive)", self.0.borrow())
        }
        */
    }
}

impl std::fmt::Display for ImlParselet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.borrow())
    }
}

impl std::ops::Deref for ImlParselet {
    type Target = Rc<RefCell<ImlParseletConfig>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ImlParselet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
