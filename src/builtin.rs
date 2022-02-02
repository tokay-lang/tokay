//! Tokay built-in functions
use crate::error::Error;
use crate::value::{Dict, Object, RefValue, Value};
use crate::vm::{Accept, Capture, Context, Reject};

use linkme::distributed_slice;

#[distributed_slice]
pub static BUILTINS: [Builtin] = [..];

// Abstraction of a built-in function
pub struct Builtin {
    pub name: &'static str,      // Function's external name
    pub signature: &'static str, // Argument signature as a string, where each argument name is separated by space
    pub func: fn(Option<&mut Context>, Vec<RefValue>) -> Result<Accept, Reject>, // Function
}

impl Builtin {
    /// Retrieve builtin by name
    pub fn get(ident: &str) -> Option<&'static Builtin> {
        for builtin in BUILTINS {
            if builtin.name == ident {
                return Some(builtin);
            }
        }

        None
    }

    /** Maps args and nargs to a builtin's signature string.

    A builtin signature string is e.g. `a b ? c d`, where `a` and `b` are mandatory parameters,
    but `c` and `d` are optional. The arguments can be provided by position (args) or by name (nargs).

    The returned vector contains all items, but optionals may be None.
    */
    pub fn map_args_and_nargs(
        &self,
        mut args: Vec<RefValue>,
        mut nargs: Option<Dict>,
    ) -> Result<Vec<RefValue>, String> {
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
                    args.push(value);
                    found_in_nargs = true;
                }
            }

            if !found_in_nargs {
                // Report required parameter which is also not found in nargs
                if required {
                    return Err(format!("{}() requires parameter '{}'", self.name, name));
                }

                args.push(RefValue::from(Value::Void));
            }

            count += 1;
        }

        //println!("args = {}, count = {}", args.len(), count);

        // Check for correct argument alignment
        if required_count >= 0 && args.len() > count {
            if count == 0 {
                return Err(format!("{}() does not accept any arguments", self.name));
            } else {
                return Err(format!(
                    "{}() does accept {} arguments only",
                    self.name, count
                ));
            }
        }

        // Check for remaining nargs not consumed
        if let Some(nargs) = nargs {
            if nargs.len() > 0 {
                return Err(format!(
                    "{}() called with unknown named argument '{}'",
                    self.name,
                    nargs.keys().nth(0).unwrap()
                ));
            }
        }

        Ok(args)
    }

    /// Directly call builtin without context and specified parameters.
    pub fn call(
        &self,
        context: Option<&mut Context>,
        args: Vec<RefValue>,
    ) -> Result<Option<RefValue>, String> {
        let args = self.map_args_and_nargs(args, None)?;

        // Call the builtin directly.
        match (self.func)(context, args) {
            Ok(Accept::Next | Accept::Hold) => Ok(None),
            Ok(Accept::Push(capture)) => Ok(Some(capture.get_value())),
            Err(Reject::Error(error)) => Err(error.message),
            other => Err(format!("Cannot handle {:?} on direct call", other)),
        }
    }
}

#[derive(Clone)]
pub struct BuiltinRef(pub &'static Builtin);

impl Object for BuiltinRef {
    fn name(&self) -> &'static str {
        "builtin"
    }

    fn repr(&self) -> String {
        format!("<{} {}>", self.name(), self.0.name)
    }

    fn is_callable(&self, _with_arguments: bool) -> bool {
        true // fixme
    }

    fn is_consuming(&self) -> bool {
        crate::utils::identifier_is_consumable(self.0.name)
    }

    fn call(
        &self,
        context: &mut Context,
        args: usize,
        nargs: Option<Dict>,
    ) -> Result<Accept, Reject> {
        // todo!!
        let args = self.0.map_args_and_nargs(context.drain(args), nargs)?;
        (self.0.func)(Some(context), args)
    }
}

impl std::fmt::Debug for BuiltinRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //write!(f, "<Builtin {}>", self.name)
        write!(f, "{}", self.0.name)
    }
}

impl From<&'static Builtin> for RefValue {
    fn from(builtin: &'static Builtin) -> Self {
        Value::Object(Box::new(BuiltinRef(builtin))).into()
    }
}

// Global built-ins

#[distributed_slice(BUILTINS)]
static CHR: Builtin = Builtin {
    name: "chr",
    signature: "i",
    func: |_context, args| {
        let i = args[0].to_usize();
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
        let c = args[0].to_string();
        if c.chars().count() != 1 {
            Error::new(
                None,
                format!(
                    "ord() expected single character, but received string of length {}",
                    c.len()
                ),
            )
            .into()
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
        if args.len() == 0 && context.is_some() {
            if let Some(capture) = context.unwrap().get_capture(0) {
                print!("{}", capture);
            }
        } else {
            for i in 0..args.len() {
                if i > 0 {
                    print!(" ");
                }

                print!("{}", args[i].to_string());
            }
        }

        print!("\n");
        Ok(Accept::Push(Capture::Value(Value::Void.into(), None, 10)))
    },
};
