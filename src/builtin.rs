//! Tokay built-in functions and parselets
use crate::compiler;
use crate::error::Error;
use crate::value::{Dict, List, RefValue, Value};
use crate::vm::{Accept, Capture, Context, Reject};

// Abstraction of a built-in function
pub struct Builtin {
    pub name: &'static str,      // Function's external name
    pub required: i8,            // Number of required arguments, -1 for dynamic parameters
    pub signature: &'static str, // Argument signature as a string, where each argument name is separated by space
    pub func: fn(&mut Context, args: Vec<Option<RefValue>>) -> Result<Accept, Reject>, // Function
}

impl Builtin {
    /// Check if builtin is consuming
    pub fn is_consumable(&self) -> bool {
        crate::utils::identifier_is_consumable(self.name)
    }

    /// Call self
    pub fn call(
        &self,
        context: &mut Context,
        args: usize,
        mut nargs: Option<Dict>,
    ) -> Result<Accept, Reject> {
        // First, collect all arguments and turn them into RefValues
        let args = context.drain(args);

        // Turn args into a mutable Vec<Option<RefValue>> initialized with all Some...
        let mut args: Vec<Option<RefValue>> = args.into_iter().map(|item| Some(item)).collect();

        // Match arguments to signature's names
        let mut count = 0;

        for name in self.signature.split(" ") {
            //println!("{:?}", name);
            if name.len() == 0 {
                continue;
            }

            if count < args.len() {
                count += 1;
                continue;
            }

            let mut found_in_nargs = false;

            if let Some(nargs) = &mut nargs {
                if let Some(value) = nargs.remove(name) {
                    args.push(Some(value));
                    found_in_nargs = true;
                }
            }

            if !found_in_nargs {
                // Report required parameter which is also not found in nargs
                if self.required > 0 && count < self.required as usize {
                    return Error::new(
                        None,
                        format!("{}() requires parameter '{}'", self.name, name),
                    )
                    .into_reject();
                }

                args.push(None);
            }

            count += 1;
        }

        //println!("args = {}, count = {}", args.len(), count);

        // Check for correct argument alignment
        if self.required >= 0 && args.len() > count {
            if count == 0 {
                return Error::new(
                    None,
                    format!("{}() does not accept any arguments", self.name),
                )
                .into_reject();
            } else {
                return Error::new(
                    None,
                    format!("{}() does accept {} arguments only", self.name, count),
                )
                .into_reject();
            }
        }

        // Check for remaining nargs not consumed
        if let Some(nargs) = nargs {
            if nargs.len() > 0 {
                return Error::new(
                    None,
                    format!(
                        "{}() called with unknown named argument '{}'",
                        self.name,
                        nargs.keys().nth(0).unwrap()
                    ),
                )
                .into_reject();
            }
        }

        //println!("{} {:?}", self.name, args);
        (self.func)(context, args)
    }
}

impl std::cmp::PartialEq for Builtin {
    // It satisfies to just compare the parselet's memory address for equality
    fn eq(&self, other: &Self) -> bool {
        self as *const Builtin as usize == other as *const Builtin as usize
    }
}

impl std::hash::Hash for Builtin {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self as *const Builtin as usize).hash(state);
    }
}

impl std::cmp::PartialOrd for Builtin {
    // It satisfies to just compare the parselet's memory address for equality
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let left = self as *const Builtin as usize;
        let right = other as *const Builtin as usize;

        left.partial_cmp(&right)
    }
}

impl std::fmt::Debug for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //write!(f, "<Builtin {}>", self.name)
        write!(f, "{}", self.name)
    }
}

inventory::collect!(Builtin);

/// Retrieve builtin by name
pub fn get(ident: &str) -> Option<&'static Builtin> {
    for builtin in inventory::iter::<Builtin> {
        if builtin.name == ident {
            return Some(builtin);
        }
    }

    None
}

// ------------------------------------------------------------------------------------------------
// Built-in standard functions
// ------------------------------------------------------------------------------------------------

inventory::submit! {
    Builtin {
        name: "ast",
        required: 1,
        signature: "emit value",
        func: |context, args| {
            let emit = args[0].as_ref().unwrap();

            let mut ret = Dict::new();
            ret.insert("emit".to_string(), emit.clone());

            let value = match &args[1] {
                Some(value) => Some(value.clone()),
                None => context
                    .collect(context.capture_start, false, true, false, 0)
                    .unwrap_or(None),
            };

            if let Some(value) = value {
                // List or Dict values are classified as child nodes
                if value.borrow().get_list().is_some() || value.borrow().get_dict().is_some() {
                    ret.insert("children".to_string(), value.clone());
                } else {
                    ret.insert("value".to_string(), value.clone());
                }
            }

            // Store positions of reader start
            ret.insert(
                "offset".to_string(),
                Value::Addr(context.reader_start.offset).into_refvalue(),
            );
            ret.insert(
                "row".to_string(),
                Value::Addr(context.reader_start.row as usize).into_refvalue(),
            );
            ret.insert(
                "col".to_string(),
                Value::Addr(context.reader_start.col as usize).into_refvalue(),
            );

            // Store positions of reader stop
            let current = context.runtime.reader.tell();

            ret.insert(
                "stop_offset".to_string(),
                Value::Addr(current.offset).into_refvalue(),
            );
            ret.insert(
                "stop_row".to_string(),
                Value::Addr(current.row as usize).into_refvalue(),
            );
            ret.insert(
                "stop_col".to_string(),
                Value::Addr(current.col as usize).into_refvalue(),
            );

            Ok(Accept::Push(
                Capture::Value(
                    Value::Dict(Box::new(ret)).into_refvalue(),
                    None,
                    10
                )
            ))
        },
    }
}

inventory::submit! {
    Builtin {
        name: "ast_print",
        required: 1,
        signature: "ast",
        func: |_, args| {
            compiler::ast::print(&args[0].as_ref().unwrap().borrow());
            Ok(Accept::Push(Capture::Value(
                Value::Void.into_refvalue(),
                None,
                10,
            )))
        },
    }
}

inventory::submit! {
    Builtin {
        name: "chr",
        required: 1,
        signature: "i",
        func: |_context, args| {
            let i = args[0].as_ref().unwrap().borrow().to_addr();
            println!("i = {}", i);

            Ok(Accept::Push(Capture::Value(
                Value::String(format!("{}", std::char::from_u32(i as u32).unwrap()))
                    .into_refvalue(),
                None,
                10,
            )))
        },
    }
}

inventory::submit! {
    Builtin {
        name: "dict",
        required: 0,
        signature: "",
        func: |_context, _args| {
            // fixme: Incomplete, concept missing.
            Ok(Accept::Push(Capture::Value(
                Value::Dict(Box::new(Dict::new())).into_refvalue(),
                None,
                10,
            )))
        },
    }
}

inventory::submit! {
    Builtin {
        name: "error",
        required: 1,
        signature: "msg collect",
        func: |context, args| {
            let msg = args[0].as_ref().unwrap();
            let collect = args[1]
                .as_ref()
                .map_or(false, |value| value.borrow().is_true());

            let mut msg = msg.borrow().to_string();

            if collect {
                if let Ok(Some(value)) =
                    context.collect(context.capture_start, false, true, false, 0)
                {
                    let value = value.borrow();

                    if let Value::String(s) = &*value {
                        msg.push_str(&format!(": '{}'", s))
                    } else {
                        msg.push_str(&format!(": {}", value.repr()))
                    }
                }
            }

            Error::new(Some(context.runtime.reader.tell()), msg).into_reject()
        },
    }
}

inventory::submit! {
    Builtin {
        name: "list",
        required: 0,
        signature: "",
        func: |_context, _args| {
            // fixme: Incomplete, concept missing.
            Ok(Accept::Push(Capture::Value(
                Value::List(Box::new(List::new())).into_refvalue(),
                None,
                10,
            )))
        },
    }
}

inventory::submit! {
    Builtin {
        name: "ord",
        required: 1,
        signature: "c",
        func: |_context, args| {
            let c = args[0].as_ref().unwrap().borrow().to_string();
            if c.chars().count() != 1 {
                Error::new(
                    None,
                    format!(
                        "ord() expected single character, but received string of length {}",
                        c.len()
                    ),
                )
                .into_reject()
            } else {
                let c = c.chars().next().unwrap();

                Ok(Accept::Push(Capture::Value(
                    Value::Addr(c as usize).into_refvalue(),
                    None,
                    10,
                )))
            }
        },
    }
}

inventory::submit! {
    Builtin {
        name: "print",
        required: -1,
        signature: "",
        func: |context, args| {
            //println!("args = {:?}", args);
            if args.len() == 0 {
                if let Some(capture) = context.get_capture(0) {
                    print!("{}", capture.borrow());
                }
            } else {
                for i in 0..args.len() {
                    if i > 0 {
                        print!(" ");
                    }

                    print!("{}", args[i].as_ref().unwrap().borrow().to_string());
                }
            }

            print!("\n");
            Ok(Accept::Push(Capture::Value(
                Value::Void.into_refvalue(),
                None,
                10,
            )))
        },
    }
}

// ------------------------------------------------------------------------------------------------
// Built-in hard-coded tokens
// ------------------------------------------------------------------------------------------------

inventory::submit! {
    Builtin {
        name: "Identifier", // Matching C-style identifiers
        required: 0,
        signature: "",
        func: |context, _args| {
            if let Some(ch) = context.runtime.reader.peek() {
                if !ch.is_alphabetic() && ch != '_' {
                    return Err(Reject::Next);
                }

                context.runtime.reader.next();
            } else {
                return Err(Reject::Next);
            }

            let mut count: usize = 1;

            while let Some(ch) = context.runtime.reader.peek() {
                if !ch.is_alphanumeric() && ch != '_' {
                    break;
                }

                context.runtime.reader.next();
                count += ch.len_utf8();
            }

            if count > 0 {
                Ok(Accept::Push(Capture::Range(
                    context.runtime.reader.capture_last(count),
                    None,
                    5,
                )))
            } else {
                Err(Reject::Next)
            }
        },
    }
}

inventory::submit! {
    Builtin {
        name: "Integer", // Matching 64-bit integers directly
        required: 0,
        signature: "",
        func: |context, _args| {
            let mut neg = false;
            let mut value: i64 = 0;

            // Sign
            if let Some(ch) = context.runtime.reader.peek() {
                if ch == '-' || ch == '+' {
                    neg = ch == '-';
                    context.runtime.reader.next();
                }
            }

            let start = context.runtime.reader.tell();

            // Digits
            while let Some(ch) = context.runtime.reader.peek() {
                if ch < '0' || ch > '9' {
                    break;
                }

                value = value * 10 + ch.to_digit(10).unwrap() as i64;
                context.runtime.reader.next();
            }

            if start.offset < context.runtime.reader.tell().offset {
                if neg {
                    value = -value;
                }

                Ok(Accept::Push(Capture::Value(
                    Value::Integer(value).into_refvalue(),
                    None,
                    5,
                )))
            } else {
                context.runtime.reader.reset(start);
                Err(Reject::Next)
            }
        },
    }
}

inventory::submit! {
    Builtin {
        name: "Word", // Matching words made of letters
        required: 0,
        signature: "min max",
        func: |context, args| {
            let min = &args[0];
            let max = &args[1];

            let mut count: usize = 0;

            while let Some(ch) = context.runtime.reader.peek() {
                if !ch.is_alphabetic() {
                    break;
                }

                context.runtime.reader.next();
                count += ch.len_utf8();
            }

            if count > 0 {
                if let Some(min) = min {
                    if count < min.borrow().to_addr() {
                        count = 0;
                    }
                }

                if let Some(max) = max {
                    if count > max.borrow().to_addr() {
                        count = 0;
                    }
                }
            }

            if count > 0 {
                Ok(Accept::Push(Capture::Range(
                    context.runtime.reader.capture_last(count),
                    None,
                    5,
                )))
            } else {
                Err(Reject::Next)
            }
        },
    }
}

// ------------------------------------------------------------------------------------------------
// Built-in string manipulation functions
// ------------------------------------------------------------------------------------------------

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
