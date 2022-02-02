//! String object
use linkme::distributed_slice;
use macros::tokay_method;

use super::{List, RefValue, Value};
use crate::builtin::{Builtin, BUILTINS};

pub fn repr(string: &str) -> String {
    let mut ret = String::with_capacity(string.len() + 2);
    ret.push('"');

    for ch in string.chars() {
        match ch {
            '\"' => ret.push_str("\\\""),
            '\n' => ret.push_str("\\n"),
            '\r' => ret.push_str("\\r"),
            '\t' => ret.push_str("\\t"),
            ch => ret.push(ch),
        }
    }

    ret.push('"');
    ret
}

impl From<&str> for RefValue {
    fn from(value: &str) -> Self {
        Value::String(value.to_string()).into()
    }
}

impl From<String> for RefValue {
    fn from(value: String) -> Self {
        Value::String(value).into()
    }
}

/*
fn get_index(&self, index: &Value) -> Result<RefValue, String> {
    let index = index.to_usize();
    if let Some(ch) = self.chars().nth(index) {
        Ok(Value::String(format!("{}", ch)).into())
    } else {
        Err(format!("Index {} beyond end of string", index))
    }
}

fn set_index(&mut self, index: &Value, value: RefValue) -> Result<(), String> {
    let index = index.to_usize();
    if index < self.len() {
        todo!();
        Ok(())
    } else {
        Err(format!("Index {} beyond end of string", index))
    }
}
*/

struct Str; // Empty struct just for the methods.

impl Str {
    tokay_method!(
        str_join(str, list) {
            let delimiter = str.to_string();
            let list = List::from(list);

            let mut ret = String::new();

            for item in list.iter() {
                if ret.len() > 0 {
                    ret.push_str(&delimiter);
                }

                ret.push_str(&item.to_string());
            }

            Ok(RefValue::from(ret))
        }
    );

    tokay_method!(
        str_lower(str) {
            Ok(RefValue::from(str.to_string().to_lowercase()))
        }
    );

    tokay_method!(
        str_replace(str, from, ?, to, n) {
            let string = str.to_string();
            let from = from.to_string();
            let to = to.to_string();

            Ok(RefValue::from(if n.is_void() {
                string.replace(&from, &to)
            } else {
                string.replacen(&from, &to, n.to_usize())
            }))
        }
    );

    tokay_method!(
        str_upper(str) {
            Ok(RefValue::from(str.to_string().to_uppercase()))
        }
    );
}

#[distributed_slice(BUILTINS)]
static STR_JOIN: Builtin = Builtin {
    name: "str_join",
    signature: "self list",
    func: Str::tokay_method_str_join,
};

#[distributed_slice(BUILTINS)]
static STR_LOWER: Builtin = Builtin {
    name: "str_lower",
    signature: "self",
    func: Str::tokay_method_str_lower,
};

#[distributed_slice(BUILTINS)]
static STR_REPLACE: Builtin = Builtin {
    name: "str_replace",
    signature: "self from ? to n",
    func: Str::tokay_method_str_replace,
};

#[distributed_slice(BUILTINS)]
static STR_UPPER: Builtin = Builtin {
    name: "str_upper",
    signature: "self",
    func: Str::tokay_method_str_upper,
};
