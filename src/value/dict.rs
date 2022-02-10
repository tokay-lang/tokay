//! Dictionary object
use linkme::distributed_slice;
use macros::tokay_method;

use std::collections::BTreeMap;

use super::{RefValue, Value};
use crate::builtin::{Builtin, BUILTINS};

// Alias for the inner dict
type InnerDict = BTreeMap<String, RefValue>;

// Dict object type
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
            if !key.chars().all(|ch| ch.is_alphabetic() || ch == '_') {
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

    tokay_method!("dict_new()", Ok(RefValue::from(Dict::new())));

    tokay_method!("dict_update(dict, other)", {
        {
            let dict = &mut *dict.borrow_mut();
            let other = &*other.borrow();

            if let Value::Dict(dict) = dict {
                if let Value::Dict(other) = other {
                    for (k, v) in other.iter() {
                        dict.insert(k.clone(), v.clone());
                    }
                } else {
                    return Err(format!(
                        "{} only accepts 'dict' as second parameter, not '{}'",
                        __function,
                        other.name()
                    ));
                }
            } else {
                return Err(format!(
                    "{} only accepts 'dict' as first parameter, not '{}'",
                    __function,
                    dict.name()
                ));
            }
        }

        Ok(dict)
    });

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

impl From<Dict> for RefValue {
    fn from(value: Dict) -> Self {
        Value::Dict(Box::new(value)).into()
    }
}

#[distributed_slice(BUILTINS)]
static DICT: Builtin = Builtin {
    name: "dict",
    signature: "",
    func: Dict::tokay_method_dict_new,
};

#[distributed_slice(BUILTINS)]
static DICT_UPDATE: Builtin = Builtin {
    name: "dict_update",
    signature: "self other",
    func: Dict::tokay_method_dict_update,
};
