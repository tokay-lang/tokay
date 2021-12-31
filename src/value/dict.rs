//! Dictionary object
use linkme::distributed_slice;
use std::collections::BTreeMap;

use super::{RefValue, Value};
use crate::builtin::{Builtin, BUILTINS};
use crate::vm::*;

type InnerDict = BTreeMap<String, RefValue>;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Dict {
    dict: InnerDict,
}

impl Dict {
    pub fn new() -> Self {
        Self {
            dict: InnerDict::new(),
        }
    }

    pub fn repr(&self) -> String {
        let mut ret = "(".to_string();

        for (key, value) in self.iter() {
            if ret.len() > 1 {
                ret.push_str(", ");
            }

            // todo: Put this into a utility function...
            if !key.chars().all(|ch| ch.is_alphabetic()) {
                ret.push('"');
                for ch in key.chars() {
                    match ch {
                        '\n' => ret.push_str("\\n"),
                        '\r' => ret.push_str("\\r"),
                        '\t' => ret.push_str("\\t"),
                        '"' => ret.push_str("\\\""),
                        _ => ret.push(ch),
                    }
                }
                ret.push('"');
            } else {
                ret.push_str(key);
            }

            ret.push_str(" => ");
            ret.push_str(&value.borrow().repr());
        }

        ret.push(')');
        ret
    }

    /*
    fn get_index(&self, index: &Value) -> Result<RefValue, String> {
        let index = index.to_string();
        if let Some(value) = self.get(&index) {
            Ok(value.clone())
        } else {
            Err(format!("Key '{}' not found", index))
        }
    }

    fn set_index(&mut self, index: &Value, value: RefValue) -> Result<(), String> {
        let index = index.to_string();
        self.insert(index, value);
        Ok(())
    }
    */
}

impl std::ops::Deref for Dict {
    type Target = InnerDict;

    fn deref(&self) -> &Self::Target {
        &self.dict
    }
}

impl std::ops::DerefMut for Dict {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.dict
    }
}

impl From<Value> for Dict {
    fn from(value: Value) -> Self {
        if let Value::Dict(dict) = value {
            *dict
        } else {
            let mut d = Dict::new();
            d.insert("#0".to_string(), value.into());
            d
        }
    }
}

impl From<&Value> for Dict {
    fn from(value: &Value) -> Self {
        if let Value::Dict(dict) = value {
            *dict.clone()
        } else {
            let mut d = Dict::new();
            d.insert("#0".to_string(), value.clone().into());
            d
        }
    }
}

impl From<Dict> for Value {
    fn from(value: Dict) -> Self {
        Value::Dict(Box::new(value))
    }
}

impl From<InnerDict> for Value {
    fn from(dict: InnerDict) -> Self {
        Value::Dict(Box::new(Dict { dict }))
    }
}

#[distributed_slice(BUILTINS)]
static DICT: Builtin = Builtin {
    name: "dict",
    signature: "",
    func: |_context, _args| {
        // fixme: Incomplete, concept missing.
        Ok(Accept::Push(Capture::Value(
            Value::Dict(Box::new(Dict::new())).into(),
            None,
            10,
        )))
    },
};

#[distributed_slice(BUILTINS)]
static DICT_UPDATE: Builtin = Builtin {
    name: "dict_update",
    signature: "self other",
    func: |_context, mut args| {
        let mut dict = args.remove(0).unwrap();
        let other = args.remove(0).unwrap();

        // If dict is not a dict, turn it into a dict
        if dict.borrow().dict().is_none() {
            let new = Dict::from(&*dict.borrow());
            dict = Value::Dict(Box::new(new)).into();
        }

        // Extend dict
        if let Value::Dict(dict) = &mut *dict.borrow_mut() {
            for (k, v) in Dict::from(&*other.borrow()).iter() {
                dict.insert(k.clone(), v.clone());
            }
        }

        Ok(Accept::Push(Capture::Value(dict, None, 10)))
    },
};
