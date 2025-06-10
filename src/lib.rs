// Tokay
// Copyright © 2025 by Jan Max Meyer, Phorward Software Technologies.
// Licensed under the MIT license. See LICENSE for more information.

/*! Tokay

    A programming language designed for ad-hoc parsing.

    Visit [https://tokay.dev](https://tokay.dev) for more information.
*/

mod _builtins; // Generated builtin registry
pub mod builtin;
pub mod compiler;
pub mod error;
pub mod reader;
#[cfg(test)]
pub mod test;
mod utils;
pub mod value;
pub mod vm;

pub use compiler::Compiler;
pub use error::Error;
pub use reader::Reader;
pub use utils::run;
pub use value::{Dict, List, Object, RefValue, Str, Value};
pub use vm::{Accept, Capture, Context, Program, Reject};

/**
 * Run a piece of code on optional input.
 */
pub fn eval(code: &str, input: &str) -> Result<RefValue, Error> {
    let mut compiler = Compiler::new();

    match compiler.compile_from_str(code) {
        Ok(None) => Ok(RefValue::from(Value::Void)),
        Ok(Some(program)) => {
            let mut reader = Reader::new(None, Box::new(std::io::Cursor::new(input.to_string())));
            let mut thread = vm::Thread::new(&program, vec![&mut reader]);

            match thread.run() {
                Ok(Some(value)) => Ok(value),
                Err(error) => Err(error),
                _ => Ok(RefValue::from(Value::Void)),
            }
        }
        Err(mut errors) => Err(errors.remove(0)),
    }
}
