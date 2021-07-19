//! Tokay built-in functions and parselets

mod _std;
mod string;
mod token;

use crate::compiler;
use crate::error::Error;
use crate::value::{Dict, RefValue};
use crate::vm::{Accept, Context, Reject};

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
        compiler::ast::identifier_is_consumable(self.name)
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
