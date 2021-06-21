//! Utility functions

use crate::compiler::Compiler;
use crate::error::Error;
use crate::reader::Reader;
use crate::value::*;

/** Turns "a string\ncontaining\tsimple escape sequences" into
"a string
containing  simple escape sequences"
*/
pub fn unescape(s: String) -> String {
    let mut chars = s.into_bytes();
    let mut len = chars.len();
    let mut i = 0;
    let mut j = 0;

    while i < len {
        if chars[j] == b'\\' {
            chars[i] = match chars[j + 1] {
                b'n' => b'\n',
                b'r' => b'\r',
                b't' => b'\t',
                c => c,
            };
            j += 2;
            len -= 1;
        } else {
            if i != j {
                chars[i] = chars[j];
            }
            j += 1;
        }

        i += 1;
    }

    chars.truncate(len);
    String::from_utf8(chars).unwrap()
}

/** Compiles and runs a source with an input.

Used mostly in tests and for quick testing purposes. */
pub fn compile_and_run(
    src: &'static str,
    input: &'static str,
    debug: bool,
) -> Result<Option<RefValue>, String> {
    let mut compiler = Compiler::new();
    compiler.debug = debug;

    match compiler.compile(Reader::new(Box::new(std::io::Cursor::new(src)))) {
        Ok(program) => program.run_from_str(input).map_err(|err| err.to_string()),
        Err(errors) => Err(errors
            .into_iter()
            .map(|err| err.to_string())
            .collect::<Vec<String>>()
            .join("\n")),
    }
}
