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

// ImlParseletInstance
// ----------------------------------------------------------------------------

/** Intermediate parselet configuration.

A parselet configuration is a model with as given constants definition.
The constants definition might be generic, which needs to be resolved first
before a parselet configuration is turned into a parselet.
*/
#[allow(dead_code)]
#[derive(Debug)]
pub(in crate::compiler) struct ImlParseletInstance {
    pub model: Rc<RefCell<ImlParseletModel>>, // Parselet base model
    pub constants: IndexMap<String, ImlValue>, // Generic signature with default configuration
    pub offset: Option<Offset>,               // Offset of definition
    pub name: Option<String>,                 // Assigned name from source (for debugging)
    pub severity: u8,                         // Capture push severity
    pub generated: bool,
}

/** Representation of parselet instance in intermediate code. */
impl ImlParseletInstance {
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
        self as *const ImlParseletInstance as usize
    }
}

impl std::fmt::Display for ImlParseletInstance {
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

impl std::cmp::PartialEq for ImlParseletInstance {
    // It satisfies to just compare the parselet's memory address for equality
    fn eq(&self, other: &Self) -> bool {
        self.model.borrow().id() == other.model.borrow().id() && self.constants == other.constants
    }
}

impl Eq for ImlParseletInstance {}

impl std::hash::Hash for ImlParseletInstance {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let model = &*self.model.borrow();
        (model as *const ImlParseletModel as usize).hash(state);
        self.constants.iter().collect::<Vec<_>>().hash(state);
    }
}

impl std::cmp::PartialOrd for ImlParseletInstance {
    // It satisfies to just compare the parselet's memory address for equality
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id().partial_cmp(&other.id())
    }
}

impl From<ImlParseletInstance> for ImlValue {
    fn from(parselet: ImlParseletInstance) -> Self {
        ImlValue::Parselet(ImlParselet::new(parselet))
    }
}

// ImlParselet
// ----------------------------------------------------------------------------

/// Shared ImlParseletInstance
#[derive(Clone, Eq, PartialEq)]
pub(in crate::compiler) struct ImlParselet {
    instance: Rc<RefCell<ImlParseletInstance>>,
}

impl ImlParselet {
    pub fn new(parselet: ImlParseletInstance) -> Self {
        Self {
            instance: Rc::new(RefCell::new(parselet)),
        }
    }

    /** Derives an intermediate parselet by another intermediate parselet (`from`).

    The namespace defines the constant configuration of a surrounding parselet (`from`),
    and extends the intermediate parselet's constant configuration, making it a derivation.

    ```tokay
    A<X>: X           # intermediate generic parselet A
    B<Y>: 'x' A<Y>    # intermediate generic parselet B using a parselet instance of A

    B<'m'> B<'n'>     # parselet instances, construct the final parselets: B<'m'>, A<'m'>, B<'n'> A<'n'>
    ```

    The function either returns a derived parselet in case it was derived, otherwise it returns
    a clone of itself.
    */
    pub fn derive(&self, from: &ImlParselet) -> Result<Self, String> {
        let instance = self.instance.borrow();
        let mut constants = instance.constants.clone();
        let mut changes = false;
        let mut required = Vec::new();

        for (name, value) in constants.iter_mut() {
            // Replace any generics until no more are open
            while let ImlValue::Generic { name, .. } = value {
                *value = from.borrow().constants.get(name).unwrap().clone();
                changes = true;
            }

            match value {
                ImlValue::SelfValue | ImlValue::SelfToken => {
                    // Replace any references of self by from
                    *value = ImlValue::Parselet(from.clone());
                    changes = true;
                }
                ImlValue::Unset => required.push(name.to_string()),
                _ => {}
            }
        }

        // Check for accepted constant configuration
        if !required.is_empty() {
            return Err(format!(
                "{} requires assignment of generic argument {}",
                instance.name.as_deref().unwrap_or("<anonymous parselet>"),
                required.join(", ")
            ));
        }

        // When there is no change, there is no derivation
        if !changes {
            return Ok(self.clone());
        }

        Ok(Self::new(ImlParseletInstance {
            model: instance.model.clone(),
            constants,
            offset: instance.offset.clone(),
            name: instance.name.clone(),
            severity: instance.severity,
            generated: instance.generated,
        }))
    }

    /** Compiles an intermediate parselet into a compiled VM parselet,
    which is part of the provided `program` and indexed by `this`. */
    pub fn compile(&self, program: &mut ImlProgram, this: usize) -> Parselet {
        let instance = self.instance.borrow();
        let model = instance.model.borrow();

        Parselet::new(
            Some(format!("{}", instance)),
            None,
            instance.severity,
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
        self.instance.borrow().hash(state);
    }
}

impl std::fmt::Debug for ImlParselet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.instance.borrow())
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
        write!(f, "{}", self.instance.borrow())
    }
}

impl std::ops::Deref for ImlParselet {
    type Target = Rc<RefCell<ImlParseletInstance>>;

    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl std::ops::DerefMut for ImlParselet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.instance
    }
}
