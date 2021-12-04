//! String object

use super::{Object, RefValue, Value};

pub type String = std::string::String;

impl Object for String {
    fn name(&self) -> &str {
        "str"
    }

    fn repr(&self) -> String {
        let mut ret = String::with_capacity(self.len() + 2);
        ret.push('"');

        for ch in self.chars() {
            match ch {
                '\"' => ret.push_str("!!"),
                '\n' => ret.push_str("\\n"),
                '\r' => ret.push_str("\\r"),
                '\t' => ret.push_str("\\t"),
                ch => ret.push(ch),
            }
        }

        ret.push('"');
        ret
    }

    fn is_true(&self) -> bool {
        self.len() > 0
    }

    fn is_string(&self) -> Option<&String> {
        Some(self)
    }

    fn to_string(&self) -> String {
        self.clone()
    }

    fn get_index(&self, index: &Value) -> Result<RefValue, String> {
        let index = index.to_addr();
        if let Some(ch) = self.chars().nth(index) {
            Ok(Value::String(format!("{}", ch)).into())
        } else {
            Err(format!("Index {} beyond end of string", index))
        }
    }

    fn set_index(&mut self, index: &Value, value: RefValue) -> Result<(), String> {
        let index = index.to_addr();
        if index < self.len() {
            todo!();
            Ok(())
        } else {
            Err(format!("Index {} beyond end of string", index))
        }
    }
}
