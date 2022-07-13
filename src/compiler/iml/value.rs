//! Intermediate value representation
use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::value::{Object, RefValue};

/** Compile-time constant value */
#[derive(Clone, Debug, PartialEq)]
pub enum ImlValue {
    Undetermined(String),
    Parselet(Rc<RefCell<ImlParselet>>),
    Value(RefValue),
}

impl ImlValue {
    pub fn unwrap(self) -> RefValue {
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
            ImlValue::Parselet(parselet) => {
                let parselet = parselet.borrow();

                if without_arguments {
                    parselet.signature.len() == 0
                        || parselet.signature.iter().all(|arg| arg.1.is_some())
                } else {
                    true
                }
            }
            ImlValue::Value(value) => value.is_callable(without_arguments),
            _ => unreachable!(),
        }
    }

    /// Check whether intermediate value represents consuming
    pub fn is_consuming(&self) -> bool {
        match self {
            ImlValue::Undetermined(ident) => crate::utils::identifier_is_consumable(ident),
            ImlValue::Parselet(parselet) => parselet.borrow().consuming.is_some(),
            ImlValue::Value(value) => value.is_consuming(),
        }
    }

    /// Check whether a value is nullable in meaning of a grammar view
    pub fn is_nullable(&self) -> bool {
        match self {
            ImlValue::Parselet(parselet) => {
                if let Some(consuming) = &parselet.borrow().consuming {
                    consuming.nullable
                } else {
                    false
                }
            }
            ImlValue::Value(value) => value.is_nullable(),
            _ => panic!("Cannot perform finalization on undetermined values"),
        }
    }
}

impl std::fmt::Display for ImlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Undetermined(s) => write!(f, "{}", s),
            Self::Parselet(p) => write!(
                f,
                "{}",
                p.borrow()
                    .name
                    .as_ref()
                    .unwrap_or(&"<unnamed parselet>".to_string())
            ),
            Self::Value(v) => write!(f, "{}", v.repr()),
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
