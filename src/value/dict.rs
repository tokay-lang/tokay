//! Dictionary object

use super::{Object, RefValue, Value};
use std::collections::BTreeMap;

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
