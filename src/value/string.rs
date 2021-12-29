//! String object
use linkme::distributed_slice;

use super::{List, Object, Value};
use crate::builtin::{Builtin, BUILTINS};

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
}

#[distributed_slice(BUILTINS)]
static STR_JOIN: Builtin = Builtin {
    name: "str_join",
    signature: "self list",
    func: |_context, args| {
        let delimiter = args[0].as_ref().unwrap().borrow().to_string();
        let list = List::from(&*args[1].as_ref().unwrap().borrow());

        let mut ret = String::new();

        for item in list {
            if ret.len() > 0 {
                ret.push_str(&delimiter);
            }

            ret.push_str(&item.borrow().to_string());
        }

        Value::String(ret).into()
    },
};

#[distributed_slice(BUILTINS)]
static STR_LOWER: Builtin = Builtin {
    name: "str_lower",
    signature: "self",
    func: |_context, args| {
        let string = args[0].as_ref().unwrap().borrow().to_string();
        Value::String(string.to_lowercase()).into()
    },
};

#[distributed_slice(BUILTINS)]
static STR_REPLACE: Builtin = Builtin {
    name: "str_replace",
    signature: "self from ? to n",
    func: |_context, args| {
        let string = args[0].as_ref().unwrap().borrow().to_string();
        let from = args[1].as_ref().unwrap().borrow().to_string();
        let to = args[2]
            .as_ref()
            .map_or("".to_string(), |value| value.borrow().to_string());

        Value::String(if let Some(n) = args[3].as_ref() {
            string.replacen(&from, &to, n.borrow().to_usize())
        } else {
            string.replace(&from, &to)
        })
        .into()
    },
};

#[distributed_slice(BUILTINS)]
static STR_UPPER: Builtin = Builtin {
    name: "str_upper",
    signature: "self",
    func: |_context, args| {
        let string = args[0].as_ref().unwrap().borrow().to_string();
        Value::String(string.to_uppercase()).into()
    },
};
