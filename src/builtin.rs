use crate::compiler;
use crate::error::Error;
use crate::value::{Dict, RefValue, Value};
use crate::vm::*;

// Abstraction of static built-in functions (builtins)
pub struct Builtin {
    name: &'static str,      // Function's external name
    required: i8,            // Number of required arguments, -1 for dynamic parameters
    signature: &'static str, // Argument signature as a string, where each argument name is separated by space
    func: fn(&mut Context, args: Vec<Option<RefValue>>) -> Result<Accept, Reject>, // Function
}

impl Builtin {
    /// Check if specific builtin is consumable by identifier
    pub fn is_consumable(&self) -> bool {
        compiler::ast::identifier_is_consumable(self.name)
    }

    // Call builtin from the VM.
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

static BUILTINS: &[Builtin] = &[
    Builtin {
        name: "ast",
        required: 1,
        signature: "emit value",
        func: |context, mut args| {
            let emit = args.remove(0).unwrap();

            let mut ret = Dict::new();
            ret.insert("emit".to_string(), emit);

            let value = args.remove(0).or_else(|| {
                // In case no value is set, collect them from the current context.
                context
                    .collect(context.capture_start, false, true, false, 0)
                    .unwrap_or(None)
            });

            if let Some(value) = value {
                // List or Dict values are classified as child nodes
                if value.borrow().get_list().is_some() || value.borrow().get_dict().is_some() {
                    ret.insert("children".to_string(), value);
                } else {
                    ret.insert("value".to_string(), value);
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

            Ok(Accept::Return(Some(
                Value::Dict(Box::new(ret)).into_refvalue(),
            )))
        },
    },
    Builtin {
        name: "ast_print",
        required: 1,
        signature: "ast",
        func: |_, mut args| {
            compiler::ast::print(&args.remove(0).unwrap().borrow());
            Ok(Accept::Push(Capture::Value(
                Value::Void.into_refvalue(),
                None,
                10,
            )))
        },
    },
    Builtin {
        name: "Integer",
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
    },
    Builtin {
        name: "Name",
        required: 0,
        signature: "",
        func: |context, _args| {
            let mut count: usize = 0;

            while let Some(ch) = context.runtime.reader.peek() {
                if !ch.is_alphanumeric() {
                    break;
                }

                context.runtime.reader.next();
                count += 1;
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
    },
    Builtin {
        name: "Cname",
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
                count += 1;
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
    },
    Builtin {
        name: "Whitespace",
        required: 0,
        signature: "",
        func: |context, _args| {
            let mut count: usize = 0;

            while let Some(ch) = context.runtime.reader.peek() {
                if !ch.is_whitespace() {
                    break;
                }

                context.runtime.reader.next();
                count += 1;
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
    },
    Builtin {
        name: "error",
        required: 1,
        signature: "msg collect",
        func: |context, mut args| {
            let msg = args.remove(0).unwrap();
            let collect = args
                .remove(0)
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
    },
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
    },
];

/// Retrieve static builtin by name
pub fn get(ident: &str) -> Option<usize> {
    for i in 0..BUILTINS.len() {
        if BUILTINS[i].name == ident {
            return Some(i);
        }
    }

    None
}

/// Check if static builtin is consuming
pub fn is_consumable(builtin: usize) -> bool {
    BUILTINS[builtin].is_consumable()
}

// Call static builtin from the VM.
pub fn call(
    builtin: usize,
    context: &mut Context,
    args: usize,
    nargs: Option<Dict>,
) -> Result<Accept, Reject> {
    let builtin = &BUILTINS[builtin];
    builtin.call(context, args, nargs)
}
