//! Token callables represented by Value::Token

use crate::ccl::Ccl;
use crate::reader::Reader;
use crate::value::Value;
use crate::vm::*;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Token {
    Void,
    EOF,
    Char(Ccl),
    BuiltinChar(fn(ch: char) -> bool),
    Chars(Ccl),
    BuiltinChars(fn(ch: char) -> bool),
    Match(String),
    Touch(String),
}

impl Token {
    pub fn any() -> Self {
        Self::Char(Ccl::new().negate())
    }

    pub fn into_value(self) -> Value {
        Value::Token(Box::new(self))
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
                            reader.capture_last(1),
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
                            reader.capture_last(1),
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
                    Ok(Accept::Push(if matches!(self, Token::Touch(_)) {
                        Capture::Range(range, None, 0)
                    } else {
                        Capture::Range(range, None, 5)
                    }))
                } else {
                    reader.reset(start);
                    Err(Reject::Next)
                }
            }
        }
    }

    pub fn is_nullable(&self) -> bool {
        match self {
            Token::Void => true,
            Token::EOF => false,
            Token::Char(ccl) | Token::Chars(ccl) => ccl.len() == 0, //True shouldn't be possible here by definition!
            Token::BuiltinChar(_) | Token::BuiltinChars(_) => true,
            Token::Match(s) | Token::Touch(s) => s.len() == 0, //True shouldn't be possible here by definition!
        }
    }
}

fn get_builtin_mapping(ident: &str) -> Option<Token> {
    Some(match ident {
        "Alphabetic" => Token::BuiltinChar(|c| c.is_alphabetic()),
        "Alphanumeric" => Token::BuiltinChar(|c| c.is_alphanumeric()),
        "Ascii" => Token::BuiltinChar(|c| c.is_ascii()),
        "AsciiAlphabetic" => Token::BuiltinChar(|c| c.is_ascii_alphabetic()),
        "AsciiAlphanumeric" => Token::BuiltinChar(|c| c.is_ascii_alphanumeric()),
        "AsciiControl" => Token::BuiltinChar(|c| c.is_ascii_control()),
        "AsciiDigit" => Token::BuiltinChar(|c| c.is_ascii_digit()),
        "AsciiGraphic" => Token::BuiltinChar(|c| c.is_ascii_graphic()),
        "AsciiHexdigit" => Token::BuiltinChar(|c| c.is_ascii_hexdigit()),
        "AsciiLowercase" => Token::BuiltinChar(|c| c.is_ascii_lowercase()),
        "AsciiPunctuation" => Token::BuiltinChar(|c| c.is_ascii_punctuation()),
        "AsciiUppercase" => Token::BuiltinChar(|c| c.is_ascii_uppercase()),
        "AsciiWhitespace" => Token::BuiltinChar(|c| c.is_ascii_whitespace()),
        "Control" => Token::BuiltinChar(|c| c.is_control()),
        "Digit" => Token::BuiltinChar(|c| c.is_digit(10)),
        "Lowercase" => Token::BuiltinChar(|c| c.is_lowercase()),
        "Numeric" => Token::BuiltinChar(|c| c.is_numeric()),
        "Uppercase" => Token::BuiltinChar(|c| c.is_uppercase()),
        "Whitespace" => Token::BuiltinChar(|c| c.is_whitespace()),
        ident if ident.len() > 1 && ident.ends_with("s") => {
            if let Some(Token::BuiltinChar(f)) = get_builtin_mapping(&ident[..ident.len() - 1]) {
                Token::BuiltinChars(f)
            } else {
                return None;
            }
        }
        _ => return None,
    })
}

/// Retrieve builtin token
pub fn get(ident: &str) -> Option<Token> {
    match ident {
        "Void" => Some(Token::Void),
        "Any" => Some(Token::any()),
        "EOF" => Some(Token::EOF),
        ident => get_builtin_mapping(ident),
    }
}
