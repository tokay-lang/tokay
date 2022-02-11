//! String object
use linkme::distributed_slice;
use macros::tokay_method;

use super::{List, RefValue, Value};
use crate::builtin::{Builtin, BUILTINS};

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Str {
    string: String,
}

impl Str {
    pub fn as_str(&self) -> &str {
        &self.string
    }

    pub fn repr(&self) -> String {
        let mut ret = String::with_capacity(self.string.len() + 2);
        ret.push('"');

        for ch in self.string.chars() {
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

    tokay_method!("str_join(str, list)", {
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
    });

    tokay_method!("str_lower(str)", {
        Ok(RefValue::from(str.to_string().to_lowercase()))
    });

    tokay_method!("str_replace(str, from, to=void, n=void)", {
        let string = str.to_string();
        let from = from.to_string();
        let to = to.to_string();

        Ok(RefValue::from(if n.is_void() {
            string.replace(&from, &to)
        } else {
            string.replacen(&from, &to, n.to_usize())
        }))
    });

    tokay_method!("str_upper(str)", {
        Ok(RefValue::from(str.to_string().to_uppercase()))
    });
}

impl std::fmt::Debug for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.string)
    }
}

impl std::fmt::Display for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }
}

impl std::ops::Deref for Str {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.string
    }
}

impl std::ops::DerefMut for Str {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.string
    }
}

impl From<String> for Str {
    fn from(string: String) -> Self {
        Str { string }
    }
}

impl From<&str> for Str {
    fn from(string: &str) -> Self {
        Str {
            string: string.to_string(),
        }
    }
}

impl From<&str> for RefValue {
    fn from(string: &str) -> Self {
        Value::Str(Str {
            string: string.to_string(),
        })
        .into()
    }
}

impl From<String> for RefValue {
    fn from(string: String) -> Self {
        Value::Str(Str { string: string }).into()
    }
}

/*
fn get_index(&self, index: &Value) -> Result<RefValue, String> {
    let index = index.to_usize();
    if let Some(ch) = self.chars().nth(index) {
        Ok(Value::Str(format!("{}", ch)).into())
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

#[distributed_slice(BUILTINS)]
static STR_JOIN: Builtin = Builtin {
    name: "str_join",
    func: Str::tokay_method_str_join,
};

#[distributed_slice(BUILTINS)]
static STR_LOWER: Builtin = Builtin {
    name: "str_lower",
    func: Str::tokay_method_str_lower,
};

#[distributed_slice(BUILTINS)]
static STR_REPLACE: Builtin = Builtin {
    name: "str_replace",
    func: Str::tokay_method_str_replace,
};

#[distributed_slice(BUILTINS)]
static STR_UPPER: Builtin = Builtin {
    name: "str_upper",
    func: Str::tokay_method_str_upper,
};
