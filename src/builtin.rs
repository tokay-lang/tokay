//! Tokay built-in functions and parselets
use crate::error::Error;
use crate::value::{Dict, RefValue, Value};
use crate::vm::{Accept, Capture, Context, Reject};

use linkme::distributed_slice;

#[distributed_slice]
pub static BUILTINS: [Builtin] = [..];

// Abstraction of a built-in function
pub struct Builtin {
    pub name: &'static str,      // Function's external name
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
        let mut required = true;
        let mut required_count = -1;

        for name in self.signature.split(" ") {
            //println!("{:?}", name);
            if name.len() == 0 {
                continue;
            }

            if name == "?" {
                assert!(required);
                required = false;
                continue;
            }

            if required {
                if required_count < 0 {
                    required_count = 1
                } else {
                    required_count += 1;
                }
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
                if required {
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
        if required_count >= 0 && args.len() > count {
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

/// Retrieve builtin by name
pub fn get(ident: &str) -> Option<&'static Builtin> {
    for builtin in BUILTINS {
        if builtin.name == ident {
            return Some(builtin);
        }
    }

    None
}

// Global built-ins

#[distributed_slice(BUILTINS)]
static CHR: Builtin = Builtin {
    name: "chr",
    signature: "i",
    func: |_context, args| {
        let i = args[0].as_ref().unwrap().borrow().to_addr();
        Ok(Accept::Push(Capture::Value(
            Value::String(format!("{}", std::char::from_u32(i as u32).unwrap())).into(),
            None,
            10,
        )))
    },
};

#[distributed_slice(BUILTINS)]
static ORD: Builtin = Builtin {
    name: "ord",
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
                Value::Addr(c as usize).into(),
                None,
                10,
            )))
        }
    },
};

#[distributed_slice(BUILTINS)]
static PRINT: Builtin = Builtin {
    name: "print",
    signature: "?",
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
        Ok(Accept::Push(Capture::Value(Value::Void.into(), None, 10)))
    },
};

// ------------------------------------------------------------------------------------------------
// Built-in hard-coded tokens
// ------------------------------------------------------------------------------------------------

#[distributed_slice(BUILTINS)]
static IDENTIFIER: Builtin = Builtin {
    name: "Identifier", // Matching C-style identifiers
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
};

#[distributed_slice(BUILTINS)]
static INTEGER: Builtin = Builtin {
    name: "Integer", // Matching 64-bit integers directly
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
                Value::Integer(value).into(),
                None,
                5,
            )))
        } else {
            context.runtime.reader.reset(start);
            Err(Reject::Next)
        }
    },
};

#[distributed_slice(BUILTINS)]
static WORD: Builtin = Builtin {
    name: "Word", // Matching words made of letters
    signature: "? min max",
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
};
