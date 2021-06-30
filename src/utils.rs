//! Utility functions

use crate::compiler::Compiler;
use crate::reader::Reader;
use crate::value::*;

/** Turns "a string\ncontaining\tsimple escape sequences" into
"a string
containing  simple escape sequences"
*/
pub fn unescape(s: &str) -> String {
    // fixme: this little parsing function should easily be implementable in Tokay itself in future.
    let mut chars = s.chars();
    let mut ret = String::with_capacity(s.len());

    while let Some(mut ch) = chars.next() {
        if ch == '\\' {
            ch = match chars.next() {
                Some(ch) => match ch {
                    'a' => '\x07',
                    'b' => '\x08',
                    'f' => '\x0c',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    'v' => '\x0b',
                    // octal / hex / unicode
                    '0'..='8' | 'x' | 'u' | 'U' => {
                        let (n, c) = match ch {
                            'u' => (4, 0),
                            'U' => (8, 0),
                            'x' => (2, 0),
                            _ => (2, 1),
                        };

                        // read code point of n digits
                        let mut code = String::with_capacity(n + c);
                        for _ in 0..n {
                            code.push(chars.next().unwrap_or_default());
                        }

                        match ch {
                            // Unicode
                            'u' | 'U' => std::char::from_u32(
                                u32::from_str_radix(&code, 16).unwrap_or_default(),
                            )
                            .unwrap_or_default(),
                            // Hex
                            'x' => u8::from_str_radix(&code, 16).unwrap_or_default() as char,
                            // Octal
                            _ => {
                                code.insert(0, ch);
                                u8::from_str_radix(&code, 8).unwrap_or_default() as char
                            }
                        }
                    }
                    ch => ch,
                },
                None => ch,
            };
        }

        ret.push(ch);
    }

    ret
}

/** Compiles and runs a source with an input.

Used mostly in tests and for quick testing purposes. */
pub fn compile_and_run(
    src: &str,
    input: &'static str,
    debug: bool,
) -> Result<Option<RefValue>, String> {
    let mut compiler = Compiler::new();
    compiler.debug = debug;

    match compiler.compile(Reader::new(Box::new(std::io::Cursor::new(src.to_owned())))) {
        Ok(program) => program.run_from_str(input).map_err(|err| err.to_string()),
        Err(errors) => Err(errors
            .into_iter()
            .map(|err| err.to_string())
            .collect::<Vec<String>>()
            .join("\n")),
    }
}
