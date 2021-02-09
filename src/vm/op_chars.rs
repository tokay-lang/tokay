use super::*;
use crate::ccl;
use crate::ccl::Ccl;
use crate::reader::Reader;

/** Character scanner.

This scanner either matches simple characters or matches ranges until a specific
character is found.
*/

#[derive(Debug)]
pub struct Chars {
    accept: Ccl,
    repeats: bool,
    silent: bool,
}

impl Chars {
    fn _new(accept: Ccl, repeats: bool, silent: bool) -> Op {
        Self {
            accept,
            repeats,
            silent,
        }
        .into_op()
    }

    pub fn new_silent(accept: Ccl) -> Op {
        Self::_new(accept, false, true)
    }

    pub fn new(accept: Ccl) -> Op {
        Self::_new(accept, false, false)
    }

    pub fn any() -> Op {
        let mut any = Ccl::new();
        any.negate();

        Self::new_silent(any)
    }

    pub fn char(ch: char) -> Op {
        Self::new_silent(ccl![ch..=ch])
    }

    pub fn span(ccl: Ccl) -> Op {
        Self::_new(ccl, true, false)
    }

    pub fn until(ch: char) -> Op {
        let mut other = ccl![ch..=ch];
        other.negate();

        Self::span(other)
    }
}

impl Scanable for Chars {
    fn scan(&self, reader: &mut Reader) -> Result<Accept, Reject> {
        let start = reader.tell();

        while let Some(ch) = reader.peek() {
            if !self.accept.test(&(ch..=ch)) {
                break;
            }

            reader.next();

            if !self.repeats {
                break;
            }
        }

        if start < reader.tell() {
            Ok(Accept::Push(Capture::Range(reader.capture_from(start), 5)))
        } else {
            reader.reset(start);
            Err(Reject::Next)
        }
    }
}

impl std::fmt::Display for Chars {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Char #todo")
    }
}
