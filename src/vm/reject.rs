//! Rejected state in Tokay VM
use super::Accept;
use crate::error::Error;

/// Represents an rejecting state within the Tokay VM.
#[derive(Debug, Clone)]
pub enum Reject {
    Next, // soft-reject, continue with next sequence
    Skip, // soft-reject, skip consumed input and continue
    Main, // hard-reject current parselet and exit to main scope ('escape'-keyword)
    Error(Box<Error>), //hard-reject with error message (runtime error)
          // todo: Exit(u32) // stop entire program with exit code
}

impl From<Error> for Reject {
    fn from(error: Error) -> Self {
        Reject::Error(Box::new(error))
    }
}

impl From<String> for Reject {
    fn from(error: String) -> Self {
        Error::new(None, error).into()
    }
}

impl From<Error> for Result<Accept, Reject> {
    fn from(error: Error) -> Self {
        Err(error.into())
    }
}
