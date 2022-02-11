//! Tokay built-in functions
use crate::value::{Dict, Object, RefValue, Value};
use crate::vm::{Accept, Context, Reject};

use linkme::distributed_slice;
use macros::tokay_function;

#[distributed_slice]
pub static BUILTINS: [Builtin] = [..];

// Abstraction of a built-in function
pub struct Builtin {
    pub name: &'static str,      // Function's external name
    pub func: fn(Option<&mut Context>, Vec<RefValue>, Option<Dict>) -> Result<Accept, Reject>, // Function
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

    /// Directly call builtin without context and specified parameters.
    pub fn call(
        &self,
        context: Option<&mut Context>,
        args: Vec<RefValue>,
    ) -> Result<Option<RefValue>, String> {
        // Call the builtin directly.
        match (self.func)(context, args, None) {
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
        let args = context.drain(args);
        (self.0.func)(Some(context), args, nargs)
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

tokay_function!("chr(i)", {
    RefValue::from(format!(
        "{}",
        std::char::from_u32(i.to_usize() as u32).unwrap()
    ))
    .into()
});

tokay_function!("ord(c)", {
    let c = c.to_string();
    if c.chars().count() != 1 {
        Err(format!(
            "{} expects a single character, but received string of length {}",
            __function,
            c.len()
        )
        .into())
    } else {
        RefValue::from(c.chars().next().unwrap() as usize).into()
    }
});

tokay_function!(
    "print(msg=void)", //fixme: print() allowed for dynamic parameters, msg is a placeholder
    {
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
        Value::Void.into()
    }
);

#[distributed_slice(BUILTINS)]
static CHR: Builtin = Builtin {
    name: "chr",
    func: tokay_function_chr,
};

#[distributed_slice(BUILTINS)]
static ORD: Builtin = Builtin {
    name: "ord",
    func: tokay_function_ord,
};

#[distributed_slice(BUILTINS)]
static PRINT: Builtin = Builtin {
    name: "print",
    func: tokay_function_print,
};
