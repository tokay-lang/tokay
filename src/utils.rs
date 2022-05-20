//! Utility functions

use crate::compiler::Compiler;
use crate::value::*;

/** Compiles a Tokay source and runs the resulting program with an input stream from a &str.

This function is mostly used internally within tests, but can also be used from outside. */
pub fn run(src: &str, input: &str) -> Result<Option<RefValue>, String> {
    let mut compiler = Compiler::new(true);

    match compiler.compile_from_str(src) {
        Ok(_) => match compiler.finalize() {
            Ok(program) => program
                .run_from_string(input.to_owned())
                .map_err(|err| err.to_string()),
            Err(errors) => Err(errors
                .into_iter()
                .map(|err| err.to_string())
                .collect::<Vec<String>>()
                .join("\n")),
        },
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
