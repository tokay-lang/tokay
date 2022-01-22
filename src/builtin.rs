//! Tokay built-in functions
use crate::error::Error;
use crate::utils;
use crate::value::{Dict, Object, RefValue, Value};
use crate::vm::{Accept, Capture, Context, Reject};

use linkme::distributed_slice;

#[distributed_slice]
pub static BUILTINS: [Builtin] = [..];

// Abstraction of a built-in function
pub struct Builtin {
    pub name: &'static str,      // Function's external name
    pub signature: &'static str, // Argument signature as a string, where each argument name is separated by space
    pub func: fn(Option<&mut Context>, Vec<Option<RefValue>>) -> Result<Accept, Reject>, // Function
}

#[derive(Clone)]
pub struct BuiltinRef(pub &'static Builtin);

impl Object for BuiltinRef {
    // Returns the callable's name.
    fn name(&self) -> &str {
        "builtin"
    }

    /// Check whether the callable accepts any arguments.
    fn is_callable(&self, _with_arguments: bool) -> bool {
        true // fixme
    }

    /// Check if builtin is consuming
    fn is_consuming(&self) -> bool {
        crate::utils::identifier_is_consumable(self.0.name)
    }

    /// Call self
    fn call(
        &self,
        context: &mut Context,
        args: usize,
        nargs: Option<Dict>,
    ) -> Result<Accept, Reject> {
        let args =
            utils::map_args_and_nargs(self.0.name, self.0.signature, context.drain(args), nargs)?;
        (self.0.func)(Some(context), args)
    }
}

impl std::fmt::Debug for BuiltinRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //write!(f, "<Builtin {}>", self.name)
        write!(f, "{}", self.0.name)
    }
}

/// Retrieve builtin by name
pub fn get(ident: &str) -> Option<BuiltinRef> {
    for builtin in BUILTINS {
        if builtin.name == ident {
            return Some(BuiltinRef(builtin));
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
        let i = args[0].as_ref().unwrap().borrow().to_usize();
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
