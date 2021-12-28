//! String object
use linkme::distributed_slice;

use super::{List, Object, Value};
use crate::builtin::{Builtin, BUILTINS};

pub type String = std::string::String;

/*
## Primitives

- `void` and `null` stand on their own
- [ ] `bool(b)`
- [ ] `int(i)`
- [ ] `float(f)`
  - What about t., t.floor, etc?

## String

- [ ] `str(s=void)` - String constructor, also converts any value to string
- [ ] `str_upper(s)` - Convert s to upper-case letter order
- [ ] `str_lower(s)` - Convert s to lower-case letter order
- [ ] `str_replace(s, f, r="", n=void)` - n-times replace f by r in s
- [ ] `str_join(s, l)` - Join list by separator, e.g. `",".join([1,2,3])  # 1,2,3`
- [ ] `str_split(s, d=" ")` - Split string by separator into list
- [ ] `str_get_attr(s, a)`, e.g `s.len`
- [ ] `str_get_index(s, i)`, e.g. `s[2]`, `s[2..8]`
- [ ] `str_set_index(s, i, v=void)`, e.g. `s[3] = "x"`, `s[3..5] = "xyz"`

## List

- [ ] `list(v=void)` - List constructor, also converts any non-list value into a list with one item
- [ ] `list_push(l, e)` - Push e to end of l
- [ ] `list_pop(l, i=void)` - Pop index i off l
- [ ] `list_insert(l, i, e)` - Insert e at index i in l
- [ ] `list_get_attr(s, a)`, e.g `l.len`
- [ ] `list_get_index(s, i)`, e.g. `l[2]`, `l[2..8]`
- [ ] `list_set_index(s, i, v=void)`, e.g. `l[3] = 42`, `l[3..5] = (7, 11, 9)`


## Dict

- [ ] `dict(l=void)` - Dict constructors
- [ ] `dict_insert(d, k, v)` - Insert v under k into d
- [ ] `dict_pop(d, k=void)` - Pop key k off d
- [ ] `list_insert(l, i, e)` - Pop index i off l
- [ ] `list_insert(l, i, e)` - Pop index i off l
*/

/*
macro_rules! builtin {
    ( $fn:ident, [ $($name:ident : $type:ty = $value:expr ),* ], $return:ty, $body:tt ) => {
        fn $fn(args: Vec<RefValue>) {
            let mut _required = true;
            $(
                {
                    let _value: Option<$type> = $value;

                    if _required && _value.is_some() {
                        _required = false;
                    }
                }

                let $name: Option<$type> = if
                if _required {

                    println!("let {}: {};", stringify!($name), stringify!($type));
                }
                else {
                    println!("let {}: {} = {};", stringify!($name), stringify!($type), stringify!($value));
                }

            )*

            println!("{} <= {}", stringify!($return), stringify!($body));
        }
    }
}
*/

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
