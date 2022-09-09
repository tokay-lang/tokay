//! Intermediate value representation
use super::*;
use crate::value::{Object, RefValue};
use std::cell::RefCell;
use std::rc::Rc;

/** Compile-time values */
#[derive(Clone, PartialEq, Eq)]
pub enum ImlValue {
    Generic(String),
    Parselet(Rc<RefCell<ImlParselet>>),
    Value(RefValue),
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
            ImlValue::Generic(ident) => crate::utils::identifier_is_consumable(ident),
            ImlValue::Parselet(parselet) => parselet.borrow().consuming,
            ImlValue::Value(value) => value.is_consuming(),
        }
    }
}

impl std::fmt::Debug for ImlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Generic(s) => write!(f, "Generic({:?})", s),
            Self::Parselet(p) => p.borrow().fmt(f),
            Self::Value(v) => v.borrow().fmt(f),
        }
    }
}

impl std::fmt::Display for ImlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Generic(s) => write!(f, "{}", s),
            Self::Parselet(p) => write!(
                f,
                "{}",
                p.borrow().name.as_deref().unwrap_or("<unnamed parselet>")
            ),
            Self::Value(v) => write!(f, "{}", v.repr()),
        }
    }
}

impl std::hash::Hash for ImlValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Generic(_) => unreachable!(),
            Self::Parselet(p) => {
                state.write_u8('p' as u8);
                p.borrow().hash(state);
            }
            Self::Value(v) => {
                state.write_u8('v' as u8);
                v.hash(state)
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
