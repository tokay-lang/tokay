//! Intermediate value representation
use super::*;
use crate::compiler::Compiler;
use crate::reader::Offset;
use crate::value::{Object, RefValue};
use indexmap::IndexMap;
use std::cell::RefCell;
use std::rc::Rc;

/** Intermediate value

Intermediate values are values that result during the compile process based on current information
from the syntax tree and symbol table information.

These can be memory locations of variables, static values, functions or values whose definition is
still pending.
*/
#[derive(Clone, PartialEq, Eq)]
pub(in crate::compiler) enum ImlValue {
    Void,            // Compile-time void
    Value(RefValue), // Compile-time static value
    Parselet {
        // Compile-time parselet instance
        parselet: Rc<RefCell<ImlParselet>>, // The parselet definition
        constants: IndexMap<String, ImlValue>, // Optional parselet instance configuation
    },
    Local(usize),  // Runtime local variable
    Global(usize), // Runtime global variable

    // Unresolved
    Generic {
        // Generic placeholder
        offset: Option<Offset>,
        name: String,
    },
    Name {
        // Unresolved name
        offset: Option<Offset>,
        name: String,
    },
    Instance {
        offset: Option<Offset>,                                  // Source offset
        target: Box<ImlValue>,                                   // Instance target
        config: Vec<(Option<Offset>, Option<String>, ImlValue)>, // Constant configuration
    },
}

impl ImlValue {
    pub fn resolve(&mut self, compiler: &mut Compiler) -> bool {
        match self {
            Self::Name { name, .. } => {
                if let Some(value) = compiler.get(&name) {
                    *self = value;
                    return true;
                }
            }
            /*
            Self::Instance {
                target,
                ..
            } if matches!(target, ImlValue::Name(_)) => {
                // Try to resolve target
                if target.resolve(compiler) {
                    // On success, try to resolve the entire instance
                    return self.resolve(compiler);
                }
            }
            Self::Instance {
                target:
                    ImlValue::Parselet {
                        parselet,
                        constants,
                    },
                config,
                offset,
            } => {
                todo!();
            }
            */
            _ => {}
        }

        false
    }

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
            Self::Name { name, .. } | Self::Generic { name, .. } => {
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
            Self::Value(v) => v.borrow().fmt(f),
            Self::Parselet { .. } => write!(f, "{}", self),
            Self::Local(addr) => write!(f, "local@{}", addr),
            Self::Global(addr) => write!(f, "global@{}", addr),
            Self::Name { name, .. } | Self::Generic { name, .. } => write!(f, "{}", name),
            _ => todo!(),
        }
    }
}

impl std::fmt::Display for ImlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Void => write!(f, "void"),
            Self::Name { name, .. } | Self::Generic { name, .. } => write!(f, "{}", name),
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
                    for (i, (name, value)) in constants.iter().enumerate() {
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
            _ => todo!(),
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
