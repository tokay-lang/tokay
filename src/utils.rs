//! Utility functions

use crate::compiler::Compiler;
use crate::reader::Reader;
use crate::value::*;

/** Compiles and runs a source with an input.

Used mostly in tests and for quick testing purposes. */
pub fn compile_and_run(src: &str, input: &'static str) -> Result<Option<Value>, String> {
    let mut compiler = Compiler::new();
    let program = compiler.compile(Reader::new(Box::new(std::io::Cursor::new(src.to_owned()))));

    match program {
        Ok(program) => program.run_from_str(input).map_err(|err| err.to_string()),
        Err(errors) => Err(errors
            .into_iter()
            .map(|err| err.to_string())
            .collect::<Vec<String>>()
            .join("\n")),
    }
}

/// Checks if an identifier defines a Tokay consumable.
pub(crate) fn identifier_is_consumable(ident: &str) -> bool {
    let ch = ident.chars().next().unwrap();
    ch.is_uppercase() || ch == '_'
}

/** Maps args and nargs to a builtin signature string.

A builtin signature string is e.g. `a b ? c d`, where `a` and `b` are mandatory parameters, and `c` and `d` are option.
The arguments can be provided by position (args) or by reference (nargs).

The returned vector contains all items, but optionals may be None.
*/
pub(crate) fn map_args_and_nargs(
    callee: &str,
    signature: &str,
    args: Vec<RefValue>,
    mut nargs: Option<Dict>,
) -> Result<Vec<Option<RefValue>>, String> {
    // Turn args into a mutable Vec<Option<RefValue>> initialized with all Some...
    let mut args: Vec<Option<RefValue>> = args.into_iter().map(|item| Some(item)).collect();

    // Match arguments to signature's names
    let mut count = 0;
    let mut required = true;
    let mut required_count = -1;

    for name in signature.split(" ") {
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
                return Err(format!("{}() requires parameter '{}'", callee, name));
            }

            args.push(None);
        }

        count += 1;
    }

    //println!("args = {}, count = {}", args.len(), count);

    // Check for correct argument alignment
    if required_count >= 0 && args.len() > count {
        if count == 0 {
            return Err(format!("{}() does not accept any arguments", callee));
        } else {
            return Err(format!("{}() does accept {} arguments only", callee, count));
        }
    }

    // Check for remaining nargs not consumed
    if let Some(nargs) = nargs {
        if nargs.len() > 0 {
            return Err(format!(
                "{}() called with unknown named argument '{}'",
                callee,
                nargs.keys().nth(0).unwrap()
            ));
        }
    }

    Ok(args)
}
