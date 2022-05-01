//! Utility functions

use crate::compiler::Compiler;
use crate::reader::Reader;
use crate::value::*;

/** Compiles a Tokay source and runs the resulting program with an input stream from a &str.

This function is mostly used internally within tests, but can also be used from outside. */
pub fn run(src: &str, input: &str) -> Result<Option<RefValue>, String> {
    let mut compiler = Compiler::new();
    let program = compiler.compile(Reader::new(Box::new(std::io::Cursor::new(src.to_owned()))));

    match program {
        Ok(program) => program
            .run_from_string(input.to_owned())
            .map_err(|err| err.to_string()),
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
