//! Intermediate value representation
use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::value::{RefValue, Value};

/** Compile-time constant value */
#[derive(Clone, Debug, PartialEq)]
pub enum ImlValue {
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
    pub fn is_callable(&self, with_arguments: bool) -> bool {
        match self {
            ImlValue::Parselet(parselet) => {
                let parselet = parselet.borrow();

                // Either without arguments and signature is empty or all arguments have default values
                (!with_arguments && (parselet.signature.len() == 0 || parselet.signature.iter().all(|arg| arg.1.is_some())))
                // or with arguments and signature exists
                    || (with_arguments && parselet.signature.len() > 0)
            }
            ImlValue::Value(value) => value.is_callable(with_arguments),
        }
    }

    /// Check whether intermediate value represents consuming
    pub fn is_consuming(&self) -> bool {
        match self {
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

impl From<Value> for ImlValue {
    fn from(value: Value) -> Self {
        Self::Value(value.into())
    }
}
