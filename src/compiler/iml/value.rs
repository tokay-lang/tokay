//! Intermediate value representation
use super::*;
use crate::value::{Object, RefValue};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/** Compile-time values */
#[derive(Clone, PartialEq, Eq)]
pub(in crate::compiler) enum ImlValue {
    Undefined(String),                  // Yet undefined value
    Value(RefValue),                    // Standard value object
    Parselet(Rc<RefCell<ImlParselet>>), // Parselet
    ParseletInstance {
        // Instance of a parselet with a constants setting
        parselet: Rc<RefCell<ImlParselet>>,
        constants: HashMap<String, ImlValue>,
    },
}

impl ImlValue {
    pub fn value(self) -> RefValue {
        if let ImlValue::Value(value) = self {
            value
        } else {
            panic!("{:?} cannot be unwrapped", self)
        }
    }

    /// Check whether intermediate value represents callable,
    /// and when its callable if with or without arguments.
    pub fn is_callable(&self, without_arguments: bool) -> bool {
        match self {
            ImlValue::Value(value) => value.is_callable(without_arguments),
            ImlValue::Parselet(parselet) | ImlValue::ParseletInstance { parselet, .. } => {
                let parselet = parselet.borrow();

                if without_arguments {
                    parselet.signature.len() == 0
                        || parselet.signature.iter().all(|arg| arg.1.is_some())
                } else {
                    true
                }
            }
            _ => unreachable!(),
        }
    }

    /// Check whether intermediate value represents consuming
    pub fn is_consuming(&self) -> bool {
        match self {
            ImlValue::Undefined(ident) => crate::utils::identifier_is_consumable(ident),
            ImlValue::Value(value) => value.is_consuming(),
            ImlValue::Parselet(parselet) | ImlValue::ParseletInstance { parselet, .. } => {
                parselet.borrow().consuming
            }
        }
    }
}

impl std::fmt::Debug for ImlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Undefined(s) => write!(f, "{}", s),
            Self::Value(v) => v.borrow().fmt(f),
            Self::Parselet(parselet) | ImlValue::ParseletInstance { parselet, .. } => {
                parselet.borrow().fmt(f)
            }
        }
    }
}

impl std::fmt::Display for ImlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn write_parselet(
            f: &mut std::fmt::Formatter<'_>,
            parselet: &ImlParselet,
            constants: Option<&HashMap<String, ImlValue>>,
        ) -> std::fmt::Result {
            write!(
                f,
                "{}",
                parselet.name.as_deref().unwrap_or("<anonymous parselet>")
            )?;

            if let Some(constants) = constants {
                write!(f, "<")?;
                for (name, value) in constants {
                    write!(f, "{}: {}", name, value)?;
                }
                write!(f, ">")?;
            }

            Ok(())
        }

        match self {
            Self::Undefined(s) => write!(f, "{}", s),
            Self::Value(v) => write!(f, "{}", v.repr()),
            Self::Parselet(parselet) => write_parselet(f, &parselet.borrow(), None),
            Self::ParseletInstance {
                parselet,
                constants,
            } => write_parselet(f, &parselet.borrow(), Some(constants)),
        }
    }
}

impl std::hash::Hash for ImlValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Undefined(_) => unreachable!(),
            Self::Value(v) => {
                state.write_u8('v' as u8);
                v.hash(state)
            }
            Self::Parselet(parselet) => {
                state.write_u8('p' as u8);
                parselet.borrow().hash(state);
            }
            Self::ParseletInstance {
                parselet,
                constants,
            } => {
                state.write_u8('i' as u8);
                parselet.borrow().hash(state);
                constants.iter().collect::<Vec<_>>().hash(state);
            }
        }
    }
}

impl From<ImlParselet> for ImlValue {
    fn from(parselet: ImlParselet) -> Self {
        Self::Parselet(Rc::new(RefCell::new(parselet)))
    }
}

impl From<RefValue> for ImlValue {
    fn from(value: RefValue) -> Self {
        Self::Value(value)
    }
}
