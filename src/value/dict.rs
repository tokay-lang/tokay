//! Dictionary object
use linkme::distributed_slice;
use std::collections::BTreeMap;

use super::{Object, RefValue, Value};
use crate::builtin::{Builtin, BUILTINS};
use crate::vm::*;

pub type Dict = BTreeMap<String, RefValue>;

impl Object for Dict {
    fn name(&self) -> &str {
        "dict"
    }

    fn repr(&self) -> String {
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

    fn is_true(&self) -> bool {
        self.len() > 0
    }

    fn is_dict(&self) -> Option<&Dict> {
        Some(self)
    }

    fn to_dict(&self) -> Dict {
        self.clone()
    }

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
        if dict.borrow().get_dict().is_none() {
            let new = dict.borrow().to_dict();
            dict = Value::Dict(Box::new(new)).into();
        }

        // Extend dict
        if let Value::Dict(dict) = &mut *dict.borrow_mut() {
            // If dict is not a dict, turn it into a dict
            for (k, v) in other.borrow().to_dict().iter() {
                dict.insert(k.clone(), v.clone());
            }
        }

        Ok(Accept::Push(Capture::Value(dict, None, 10)))
    },
};
