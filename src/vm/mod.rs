//! Tokay virtual machine
mod capture;
mod context;
mod op;
mod program;
mod runtime;

use crate::error::Error;
use crate::value::{RefValue, Value};
pub use capture::*;
pub use context::*;
pub(crate) use op::*;
pub use program::*;
pub use runtime::*;
extern crate self as tokay;

// --- Accept ------------------------------------------------------------------

/// Representing the Ok-value result on a branched run of the VM.
#[derive(Debug, Clone)]
pub enum Accept {
    Next,                     // soft-accept, run next instructions at incremented ip
    Hold,                     // soft-accept, run next instruction at current ip
    Push(Capture),            // soft-accept, push a capture (also 'push'-keyword)
    Repeat(Option<RefValue>), // hard-accept, repeat entire parselet ('repeat'-keyword)
    Return(Option<RefValue>), // hard-accept, return/accept entire parselet ('return/accept'-keyword)
}

impl Accept {
    // Helper function, turning an Accept into an Accept::Push() with a Capture and a given severity.
    pub fn into_push(self, severity: u8) -> Accept {
        match self {
            Self::Next | Self::Hold => Self::Push(Capture::Empty),
            Self::Push(mut capture) => {
                if capture.get_severity() > severity {
                    capture.set_severity(severity);
                }
                Self::Push(capture)
            }
            Self::Repeat(value) | Self::Return(value) => {
                if let Some(value) = value {
                    Self::Push(Capture::Value(value, None, severity))
                } else {
                    Self::Push(Capture::Empty)
                }
            }
        }
    }

    // Helper function, extracts a contained RefValue from the Accept.
    pub fn into_refvalue(self) -> RefValue {
        match self {
            Self::Push(capture) => capture.get_value(),
            Self::Repeat(Some(value)) | Self::Return(Some(value)) => value,
            _ => tokay::value!(void),
        }
    }
}

impl From<RefValue> for Result<Accept, Reject> {
    fn from(value: RefValue) -> Self {
        Ok(Accept::Push(value.into()))
    }
}

impl From<Value> for Result<Accept, Reject> {
    fn from(value: Value) -> Self {
        Ok(Accept::Push(RefValue::from(value).into()))
    }
}

// --- Reject ------------------------------------------------------------------

/// Representing the Err-value result on a branched run of the VM.
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
