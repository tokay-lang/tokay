use crate::ccl::Ccl;
use crate::reader::{Reader, Range};
use crate::value::{Value, RefValue};
use crate::utils;
use crate::ccl;



/** Capture value */
#[derive(Debug, Clone, PartialEq)]
pub enum Capture {
    Empty,                      // Empty capture without any further value
    Range(Range, u8),           // Captured length with a severity
    Value(RefValue, u8),        // Value with severity
}


/** Token trait */
pub trait Token: std::fmt::Debug {
    fn read(&self, reader: &mut Reader) -> Option<Capture>;

    fn into_box(self) -> Box<dyn Token>
        where Self: std::marker::Sized + 'static
    {
        Box::new(self)
    }
}


// ----------------------------------------------------------------------------
// Any - Match any character except EOF
// ----------------------------------------------------------------------------

pub struct Any;

impl Any {
    pub fn new() -> Box<dyn Token> {
        Self.into_box()
    }
}

impl Token for Any {

    fn read(&self, reader: &mut Reader) -> Option<Capture> {
        if let Some(_) = reader.next() {
            return Some(Capture::Range(reader.capture_last(1), 0))
        }

        None
    }
}

impl std::fmt::Debug for Any {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ".")
    }
}

#[test]
fn test_any() {
    let mut r = Reader::new(Box::new(std::io::Cursor::new("x".to_string())));
    let t = Any::new();
    assert_eq!(t.read(&mut r), Some(Capture::Range(0..1, 0)));
}


// ----------------------------------------------------------------------------
// Match/Touch - Match exact string
// ----------------------------------------------------------------------------

pub struct Match {
    string: String,
    severity: u8
}

impl Match {
    pub fn new(string: &str) -> Box<dyn Token> {
        Self{
            string: string.to_string(),
            severity: 1
        }.into_box()
    }

    pub fn new_touch(string: &str) -> Box<dyn Token> {
        Self{
            string: string.to_string(),
            severity: 0
        }.into_box()
    }
}

impl Token for Match {

    fn read(&self, reader: &mut Reader) -> Option<Capture> {
        for ch in self.string.chars() {
            if let Some(c) = reader.next() {
                if c != ch {
                    return None;
                }
            }
            else {
                return None;
            }
        }

        Some(Capture::Range(
            reader.capture_last(self.string.len()), self.severity)
        )
    }
}

impl std::fmt::Debug for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.severity == 0 {
            write!(f, "'{}'", self.string)
        } else {
            write!(f, "\"{}\"", self.string)
        }
    }
}

#[test]
fn test_match() {
    let mut r = Reader::new(Box::new(std::io::Cursor::new("Hello".to_string())));

    let t = Match::new("Hello");
    assert_eq!(t.read(&mut r), Some(Capture::Range(0..5, 1)));

    r.reset(0);

    let t = Match::new_touch("Hello");
    assert_eq!(t.read(&mut r), Some(Capture::Range(0..5, 0)));
}


// ----------------------------------------------------------------------------
// Char - Match single character from a range of chars
// ----------------------------------------------------------------------------

pub struct Char {
    accept: Ccl
}

impl Char {
    pub fn new(accept: Ccl) -> Box<dyn Token> {
        Self{
            accept
        }.into_box()
    }
}

impl Token for Char {
    fn read(&self, reader: &mut Reader) -> Option<Capture> {
        if let Some(c) = reader.next() {
            if self.accept.test(&(c..=c)) {
                return Some(Capture::Range(reader.capture_last(1), 0))
            }
        }

        None
    }
}

impl std::fmt::Debug for Char {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.accept)
    }
}


#[test]
fn test_char() {
    let mut r = Reader::new(Box::new(std::io::Cursor::new("abC".to_string())));

    let t = Char::new(ccl!['a'..='z']);
    assert_eq!(t.read(&mut r), Some(Capture::Range(0..1, 0)));
    assert_eq!(t.read(&mut r), Some(Capture::Range(1..2, 0)));
    assert_eq!(t.read(&mut r), None);
}


// ----------------------------------------------------------------------------
// Chars - Match range of chars from a given set
// ----------------------------------------------------------------------------

pub struct Chars {
    accept: Ccl
}

impl Chars {
    pub fn new(accept: Ccl) -> Box<dyn Token> {
        Self{
            accept
        }.into_box()
    }
}

impl Token for Chars {
    fn read(&self, reader: &mut Reader) -> Option<Capture> {
        let start = reader.tell();

        while let Some(c) = reader.peek() {
            if !self.accept.test(&(c..=c)) {
                break;
            }

            reader.next();
        }

        if start < reader.tell() {
            Some(Capture::Range(reader.capture_from(start), 0))
        }
        else {
            None
        }
    }
}

impl std::fmt::Debug for Chars {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}+", self.accept)
    }
}


#[test]
fn test_chars() {
    let mut r = Reader::new(Box::new(std::io::Cursor::new("abcC".to_string())));

    let t = Chars::new(ccl!['a'..='z']);
    assert_eq!(t.read(&mut r), Some(Capture::Range(0..3, 0)));
    assert_eq!(t.read(&mut r), None);
}

// ----------------------------------------------------------------------------
// UntilChar - Match any char until a given char is found, optionally escaped
// ----------------------------------------------------------------------------

pub struct UntilChar {
    until: char,
    escape: Option<char>
}

impl UntilChar {
    pub fn new(until: char, escape: Option<char>) -> Box<dyn Token> {
        Self{
            until,
            escape
        }.into_box()
    }
}

impl Token for UntilChar {
    fn read(&self, reader: &mut Reader) -> Option<Capture> {
        let start = reader.tell();
        let mut escapes = false;

        while let Some(c) = reader.peek() {
            if c == self.until {
                if escapes {
                    let s = reader.extract(&(start..reader.tell()));
                    let s = utils::unescape(s);

                    return Some(
                        Capture::Value(Value::String(s).into_ref(), 1)
                    )
                }

                return Some(Capture::Range(reader.capture_from(start), 1))
            }
            else if self.escape.is_some() && c == self.escape.unwrap() {
                reader.next();

                if reader.peek().is_some() {
                    escapes = true;
                }
            }

            reader.next();
        }

        None
    }
}

impl std::fmt::Debug for UntilChar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.escape.is_some() {
            write!(f, "Until('{}', Escape='{}')",
                self.until, self.escape.unwrap())
        } else {
            write!(f, "Until('{}')", self.until)
        }
    }
}
