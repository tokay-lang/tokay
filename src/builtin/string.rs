//! Built-in string manipulation functions

use super::Builtin;
use crate::value::Value;

inventory::submit! {
    Builtin {
        name: "str_join",
        required: 2,
        signature: "self list",
        func: |_context, args| {
            let delimiter = args[0].as_ref().unwrap().borrow().to_string();
            let list = args[1].as_ref().unwrap().borrow().to_list();

            let mut ret = String::new();

            for item in list {
                if ret.len() > 0 {
                    ret.push_str(&delimiter);
                }

                ret.push_str(&item.borrow().to_string());
            }

            Value::String(ret).into_accept_push_capture()
        },
    }
}

inventory::submit! {
    Builtin {
        name: "str_lower",
        required: 1,
        signature: "self",
        func: |_context, args| {
            let string = args[0].as_ref().unwrap().borrow().to_string();
            Value::String(string.to_lowercase()).into_accept_push_capture()
        },
    }
}

inventory::submit! {
    Builtin {
        name: "str_replace",
        required: 2,
        signature: "self from to n",
        func: |_context, args| {
            let string = args[0].as_ref().unwrap().borrow().to_string();
            let from = args[1].as_ref().unwrap().borrow().to_string();
            let to = args[2]
                .as_ref()
                .map_or("".to_string(), |value| value.borrow().to_string());

            Value::String(if let Some(n) = args[3].as_ref() {
                string.replacen(&from, &to, n.borrow().to_addr())
            } else {
                string.replace(&from, &to)
            })
            .into_accept_push_capture()
        },
    }
}
inventory::submit! {
    Builtin {
        name: "str_upper",
        required: 1,
        signature: "self",
        func: |_context, args| {
            let string = args[0].as_ref().unwrap().borrow().to_string();
            Value::String(string.to_uppercase()).into_accept_push_capture()
        },
    }
}
