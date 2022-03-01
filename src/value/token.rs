//! Token callables represented by Value::Token
use macros::tokay_token;

use super::{BoxedObject, Dict, Object, RefValue};

use crate::reader::Reader;
use crate::value;
use crate::vm::*;
use charclass::{charclass, CharClass};

#[derive(Debug, Clone)]
pub enum Token {
    Void,                               // Matches the empty word
    EOF,                                // Matches End of File
    Char(CharClass),                    // Matches one character from a character class
    BuiltinChar(fn(ch: char) -> bool),  // Matches one character from a callback function
    Chars(CharClass),                   // Matches multiple characters from a character class
    BuiltinChars(fn(ch: char) -> bool), // Matches multiple characters from a callback function
    Match(String),                      // Match a string
    Touch(String),                      // Match a string with zero severity
}

impl Token {
    /// Retrieve builtin token
    pub fn builtin(ident: &str) -> Option<Token> {
        fn builtin_ccl(ident: &str) -> Option<Token> {
            Some(match ident {
                "Alphabetic" => Token::BuiltinChar(|c| c.is_alphabetic()),
                "Alphanumeric" => Token::BuiltinChar(|c| c.is_alphanumeric()),
                "Ascii" => Token::BuiltinChar(|c| c.is_ascii()),
                "AsciiAlphabetic" => Token::Char(charclass!['A' => 'Z', 'a' => 'z']),
                "AsciiAlphanumeric" => Token::Char(charclass!['A' => 'Z', 'a' => 'z', '0' => '9']),
                "AsciiControl" => Token::BuiltinChar(|c| c.is_ascii_control()),
                "AsciiDigit" => Token::Char(charclass!['0' => '9']),
                "AsciiGraphic" => Token::Char(charclass!['!' => '~']),
                "AsciiHexdigit" => Token::Char(charclass!['0' => '9', 'A' => 'F', 'a' => 'f']),
                "AsciiLowercase" => Token::Char(charclass!['a' => 'z']),
                "AsciiPunctuation" => Token::BuiltinChar(|c| c.is_ascii_punctuation()),
                "AsciiUppercase" => Token::Char(charclass!['A' => 'Z']),
                "AsciiWhitespace" => Token::Char(charclass!['A' => 'Z', 'a' => 'z']),
                "Control" => Token::BuiltinChar(|c| c.is_control()),
                "Digit" => Token::BuiltinChar(|c| c.is_digit(10)),
                "Lowercase" => Token::BuiltinChar(|c| c.is_lowercase()),
                "Numeric" => Token::BuiltinChar(|c| c.is_numeric()),
                "Uppercase" => Token::BuiltinChar(|c| c.is_uppercase()),
                "Whitespace" => Token::BuiltinChar(|c| c.is_whitespace()),

                // Any identifier attached with an "s" will be checked for Token+
                ident if ident.len() > 1 && ident.ends_with("s") => {
                    if let Some(Token::BuiltinChar(f)) = builtin_ccl(&ident[..ident.len() - 1]) {
                        Token::BuiltinChars(f)
                    } else {
                        return None;
                    }
                }
                _ => return None,
            })
        }

        match ident {
            "Any" => Some(Token::any()),
            "EOF" => Some(Token::EOF),
            "Void" => Some(Token::Void),
            ident => builtin_ccl(ident),
        }
    }

    pub fn any() -> Self {
        Self::Char(CharClass::new().negate())
    }

    pub fn read(&self, reader: &mut Reader) -> Result<Accept, Reject> {
        match self {
            Token::Void => Ok(Accept::Push(Capture::Empty)),
            Token::EOF => {
                if let None = reader.peek() {
                    Ok(Accept::Next)
                } else {
                    Err(Reject::Next)
                }
            }
            Token::Char(ccl) => {
                if let Some(ch) = reader.peek() {
                    if ccl.test(&(ch..=ch)) {
                        reader.next();
                        return Ok(Accept::Push(Capture::Range(
                            reader.capture_last(ch.len_utf8()),
                            None,
                            5,
                        )));
                    }
                }

                Err(Reject::Next)
            }
            Token::BuiltinChar(f) => {
                if let Some(ch) = reader.peek() {
                    if f(ch) {
                        reader.next();
                        return Ok(Accept::Push(Capture::Range(
                            reader.capture_last(ch.len_utf8()),
                            None,
                            5,
                        )));
                    }
                }

                Err(Reject::Next)
            }
            Token::Chars(ccl) => {
                let start = reader.tell();

                while let Some(ch) = reader.peek() {
                    if !ccl.test(&(ch..=ch)) {
                        break;
                    }

                    reader.next();
                }

                let range = reader.capture_from(&start);

                if range.len() > 0 {
                    Ok(Accept::Push(Capture::Range(range, None, 5)))
                } else {
                    reader.reset(start);
                    Err(Reject::Next)
                }
            }
            Token::BuiltinChars(f) => {
                let start = reader.tell();

                while let Some(ch) = reader.peek() {
                    if !f(ch) {
                        break;
                    }

                    reader.next();
                }

                let range = reader.capture_from(&start);

                if range.len() > 0 {
                    Ok(Accept::Push(Capture::Range(range, None, 5)))
                } else {
                    reader.reset(start);
                    Err(Reject::Next)
                }
            }
            Token::Match(string) | Token::Touch(string) => {
                let start = reader.tell();

                for ch in string.chars() {
                    if let Some(c) = reader.peek() {
                        if c != ch {
                            break;
                        }
                    } else {
                        break;
                    }

                    reader.next();
                }

                let range = reader.capture_from(&start);

                if range.len() == string.len() {
                    Ok(Accept::Push(Capture::Range(
                        range,
                        None,
                        if matches!(self, Token::Touch(_)) {
                            0
                        } else {
                            5
                        },
                    )))
                } else {
                    reader.reset(start);
                    Err(Reject::Next)
                }
            }
        }
    }
}

impl Object for Token {
    fn name(&self) -> &'static str {
        "token"
    }

    /*
    fn repr(&self) -> String {
        match self {
            Token::Void => "Void".to_string(),
            Token::EOF => "EOF".to_string(),
            Token::Char(ccl) => format!("{:?}", ccl),
            Token::Chars(ccl) => format!("{:?}+", ccl),
            Token::BuiltinChar(_) | Token::BuiltinChars(_) => "\"<token builtin fn>\n".to_string(),
            Token::Touch(s) => format!("'{}'", s),
            Token::Match(s) => format!("''{}''", s),
        }
    }
    */

    fn is_callable(&self, with_arguments: bool) -> bool {
        !with_arguments // Tokens don't support arguments
    }

    fn is_consuming(&self) -> bool {
        true // Tokens always consume!
    }

    fn is_nullable(&self) -> bool {
        match self {
            Token::Void => true,
            Token::EOF => false,
            Token::Char(ccl) | Token::Chars(ccl) => ccl.len() == 0, //True shouldn't be possible here by definition!
            Token::BuiltinChar(_) | Token::BuiltinChars(_) => true,
            Token::Match(s) | Token::Touch(s) => s.len() == 0, //True shouldn't be possible here by definition!
        }
    }

    fn call(
        &self,
        context: &mut Context,
        args: usize,
        nargs: Option<Dict>,
    ) -> Result<Accept, Reject> {
        assert!(args == 0 && nargs.is_none());
        self.read(context.runtime.reader)
    }
}

impl From<Token> for RefValue {
    fn from(token: Token) -> Self {
        RefValue::from(Box::new(token) as BoxedObject)
    }
}

// Hard-coded Tokens are builtins, but they are consumable.

// Matching C-style identifiers
tokay_token!("Identifier", {
    if let Some(ch) = context.runtime.reader.peek() {
        if !ch.is_alphabetic() && ch != '_' {
            return Err(Reject::Next);
        }

        context.runtime.reader.next();
    } else {
        return Err(Reject::Next);
    }

    let mut count: usize = 1;

    while let Some(ch) = context.runtime.reader.peek() {
        if !ch.is_alphanumeric() && ch != '_' {
            break;
        }

        context.runtime.reader.next();
        count += ch.len_utf8();
    }

    if count > 0 {
        Ok(Accept::Push(Capture::Range(
            context.runtime.reader.capture_last(count),
            None,
            5,
        )))
    } else {
        Err(Reject::Next)
    }
});

// Matching 64-bit integers directly
tokay_token!("Integer", {
    let mut neg = false;
    let mut value: i64 = 0;

    // Sign
    if let Some(ch) = context.runtime.reader.peek() {
        if ch == '-' || ch == '+' {
            neg = ch == '-';
            context.runtime.reader.next();
        }
    }

    let start = context.runtime.reader.tell();

    // Digits
    while let Some(ch) = context.runtime.reader.peek() {
        if ch < '0' || ch > '9' {
            break;
        }

        value = value * 10 + ch.to_digit(10).unwrap() as i64;
        context.runtime.reader.next();
    }

    if start.offset < context.runtime.reader.tell().offset {
        if neg {
            value = -value;
        }

        Ok(Accept::Push(Capture::Value(
            RefValue::from(value!(value)),
            None,
            5,
        )))
    } else {
        context.runtime.reader.reset(start);
        Err(Reject::Next)
    }
});

// Words, optionally with limited length
tokay_token!("Word(min=void max=void)", {
    let min = if min.is_void() {
        None
    } else {
        Some(min.to_usize())
    };

    let max = if max.is_void() {
        None
    } else {
        Some(max.to_usize())
    };

    let mut count: usize = 0;

    while let Some(ch) = context.runtime.reader.peek() {
        if !ch.is_alphabetic() {
            break;
        }

        context.runtime.reader.next();
        count += ch.len_utf8();
    }

    if count > 0 {
        if let Some(min) = min {
            if count < min {
                count = 0;
            }
        }

        if let Some(max) = max {
            if count > max {
                count = 0;
            }
        }
    }

    if count > 0 {
        Ok(Accept::Push(Capture::Range(
            context.runtime.reader.capture_last(count),
            None,
            5,
        )))
    } else {
        Err(Reject::Next)
    }
});
