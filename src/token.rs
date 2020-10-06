use crate::ccl::Ccl;
use crate::reader::{Reader, Range};
use crate::value::RefValue;

/**
    Represents the result of a token capture
*/
#[derive(Debug, Clone)]
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


pub struct Char {
    accept: Ccl
}

impl Char {
    pub fn new(accept: Ccl) -> Box<dyn Token> {
        Self{
            accept: accept.clone()
        }.into_box()
    }
}

impl Token for Char {
    fn read(&self, reader: &mut Reader) -> Option<Capture> {
        if let Some(c) = reader.next() {
            if self.accept.test(&(c..=c)) {
                return Some(Capture::Range(reader.capture_last(1), 1))
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
fn test_none() {
    let mut r = Reader::new(Box::new(std::io::Cursor::new("".to_string())));
    let t = Empty::new();
    assert!(matches!(t.read(&mut r), Some(Capture::Empty)));
}

#[test]
fn test_any() {
    let mut r = Reader::new(Box::new(std::io::Cursor::new("x".to_string())));
    let t = Any::new();
    assert!(matches!(t.read(&mut r), Some(Capture::Range(0..1, 0))));
}


#[test]
fn test_match() {
    let mut r = Reader::new(Box::new(std::io::Cursor::new("Hello".to_string())));

    let t = Match::new("Hello");
    assert!(matches!(t.read(&mut r), Some(Capture::Range(0..5, 1))));

    r.reset(0);

    let t = Match::new_touch("Hello");
    assert!(matches!(t.read(&mut r), Some(Capture::Range(0..5, 0))));

}