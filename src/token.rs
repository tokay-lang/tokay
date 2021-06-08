use crate::ccl;
use crate::ccl::Ccl;
use crate::reader::Reader;
use crate::value::Value;
use crate::vm::*;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Token {
    Void,
    Any,
    EOF,
    Char(Ccl),
    Chars(Ccl),
    Match(String),
    Touch(String),
}

impl Token {
    // Repeat any char until given stop character
    pub fn chars_until(ch: char) -> Self {
        let mut ccl = ccl![ch..=ch];
        ccl.negate();
        Self::Chars(ccl)
    }

    pub fn char_except(ch: char) -> Self {
        let mut ccl = ccl![ch..=ch];
        ccl.negate();
        Self::Char(ccl)
    }

    pub fn into_value(self) -> Value {
        Value::Token(Box::new(self))
    }

    pub fn read(&self, reader: &mut Reader) -> Result<Accept, Reject> {
        match self {
            Token::Void => Ok(Accept::Push(Capture::Empty)),
            Token::Any => {
                if let Some(_) = reader.next() {
                    return Ok(Accept::Push(Capture::Range(reader.capture_last(1), None, 5)));
                }

                Err(Reject::Next)
            }
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
                        return Ok(Accept::Push(Capture::Range(reader.capture_last(1), None, 5)));
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
            Token::Any | Token::EOF => false,
            Token::Char(ccl) | Token::Chars(ccl) => ccl.len() == 0, //True shouldn't be possible here by definition!
            Token::Match(s) | Token::Touch(s) => s.len() == 0, //True shouldn't be possible here by definition!
        }
    }
}
