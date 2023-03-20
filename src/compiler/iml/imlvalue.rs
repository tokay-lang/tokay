//! Intermediate value representation
use super::*;
use crate::value::{Object, RefValue};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/** Intermediate value */
#[derive(Clone, PartialEq, Eq)]
pub(in crate::compiler) enum ImlValue {
    Void,              // Compile-time void
    Unknown(String),   // Compile-time unknown identifier
    Undefined(String), // Compile-time known but undefined identifier (used in generic parselets)
    Value(RefValue),   // Compile-time static value
    Parselet {
        // Compile-time parselet instance
        parselet: Rc<RefCell<ImlParselet>>, // The parselet definition
        constants: HashMap<String, ImlValue>, // Optional parselet instance configuation
    },
    Local(usize),  // Runtime local value
    Global(usize), // Runtime global value
}

impl ImlValue {
    pub fn into_refvalue(self) -> RefValue {
        if let Self::Value(value) = self {
            value
        } else {
            panic!("{:?} cannot be unwrapped", self)
        }
    }

    /// Check whether intermediate value represents callable,
    /// and when its callable if with or without arguments.
    pub fn is_callable(&self, without_arguments: bool) -> bool {
        match self {
            Self::Value(value) => value.is_callable(without_arguments),
            Self::Parselet { parselet, .. } => {
                let parselet = parselet.borrow();

                if without_arguments {
                    parselet.signature.len() == 0
                        || parselet
                            .signature
                            .iter()
                            .all(|arg| !matches!(arg.1, Self::Void))
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    /// Check whether intermediate value represents consuming
    pub fn is_consuming(&self) -> bool {
        match self {
            Self::Unknown(name) | Self::Undefined(name) => {
                crate::utils::identifier_is_consumable(name)
            }
            Self::Value(value) => value.is_consuming(),
            Self::Parselet { parselet, .. } => parselet.borrow().consuming,
            _ => false,
        }
    }
}

impl std::fmt::Debug for ImlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Void => write!(f, "void"),
            Self::Unknown(name) | Self::Undefined(name) => write!(f, "{}", name),
            Self::Value(v) => v.borrow().fmt(f),
            Self::Parselet { .. } => write!(f, "{}", self),
            Self::Local(addr) => write!(f, "local@{}", addr),
            Self::Global(addr) => write!(f, "global@{}", addr),
        }
    }
}

impl std::fmt::Display for ImlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Void => write!(f, "void"),
            Self::Unknown(name) | Self::Undefined(name) => write!(f, "{}", name),
            Self::Value(value) => write!(f, "{}", value.repr()),
            Self::Parselet {
                parselet,
                constants,
            } => {
                write!(
                    f,
                    "{}",
                    parselet
                        .borrow()
                        .name
                        .as_deref()
                        .unwrap_or("<anonymous parselet>")
                )?;

                if !constants.is_empty() {
                    write!(f, "<")?;
                    for (name, value) in constants {
                        write!(f, "{}: {}", name, value)?;
                    }
                    write!(f, ">")?;
                }

                Ok(())
            }
            Self::Local(addr) => write!(f, "local@{}", addr),
            Self::Global(addr) => write!(f, "global@{}", addr),
        }
    }
}

impl std::hash::Hash for ImlValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Value(v) => {
                state.write_u8('v' as u8);
                v.hash(state)
            }
            Self::Parselet {
                parselet,
                constants,
            } => {
                state.write_u8('p' as u8);
                parselet.borrow().hash(state);
                constants.iter().collect::<Vec<_>>().hash(state);
            }
            other => unreachable!("{:?} is unhashable", other),
        }
    }
}

impl From<RefValue> for ImlValue {
    fn from(value: RefValue) -> Self {
        Self::Value(value)
    }
}
