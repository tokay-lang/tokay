//! Implementation of an error object that can occur during Tokay's program compilation or execution

use crate::reader::Offset;
use crate::vm::{Accept, Reject};

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    offset: Option<Offset>,
    message: String,
}

impl Error {
    /// Creates a new Reject::Error with a message.
    pub fn new(offset: Option<Offset>, message: String) -> Error {
        Error { offset, message }
    }

    /// Turns an Error into Result<Reject::Error<Box>>
    pub fn into_reject(self) -> Result<Accept, Reject> {
        Err(Reject::Error(Box::new(self)))
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
