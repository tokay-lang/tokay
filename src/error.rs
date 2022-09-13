//! Implementation of an error object that can occur during Tokay's program compilation or execution
use crate::reader::Offset;
use crate::value::{Object, Str};
extern crate self as tokay;
use tokay_macros::tokay_function;

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub offset: Option<Offset>,
    pub message: String,
}

impl Error {
    /// Creates a new Error object with a message.
    pub fn new(offset: Option<Offset>, message: String) -> Error {
        Error { offset, message }
    }

    /// Attaches position information to an error message when not already present
    pub fn patch_offset(&mut self, offset: Offset) {
        if let None = self.offset {
            self.offset = Some(offset);
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(offset) = self.offset {
            write!(
                f,
                "Line {}, column {}: {}",
                offset.row, offset.col, self.message
            )
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::new(None, error)
    }
}

impl From<&str> for Error {
    fn from(error: &str) -> Self {
        error.to_string().into()
    }
}

tokay_function!("error(msg, collect=false)", {
    let context = context.unwrap();
    let mut msg = msg.to_string();

    if collect.is_true() {
        if let Ok(Some(value)) =
            context.collect(context.capture_start, false, true, false, 0, false)
        {
            let value = value.borrow();

            if let Some(s) = value.object::<Str>() {
                msg.push_str(&format!(": '{}'", s))
            } else {
                msg.push_str(&format!(": {}", value.repr()))
            }
        }
    }

    Error::new(Some(context.runtime.reader.tell()), msg).into()
});

#[test]
fn test_error() {
    assert_eq!(
        crate::run("'a'+ error(\"Error!\")", "aaa"),
        Err("Line 1, column 4: Error!".to_string())
    );
}
