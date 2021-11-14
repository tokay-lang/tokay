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
