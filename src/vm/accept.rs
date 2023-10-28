//! Accepting state in Tokay VM
use super::{Capture, Reject};
use crate::value::{RefValue, Value};
extern crate self as tokay;

/// Representing an accepting state within the Tokay VM.
#[derive(Debug, Clone)]
pub enum Accept {
    Next,            // soft-accept, run next instructions at incremented ip
    Hold,            // soft-accept, run next instruction at current ip
    Repeat,          // hard-accept, repeat parselet on current position ('repeat'-keyword)
    Push(Capture),   // soft-accept, push a capture (also 'push'-keyword)
    Return(Capture), // hard-accept, return parselet ('return/accept'-keywords)
}

impl Accept {
    // Helper function, turning an Accept into an Accept::Push() with a Capture and a given severity.
    pub fn into_push(self, severity: u8) -> Accept {
        match self {
            Self::Next | Self::Hold => Self::Push(Capture::Empty),
            Self::Push(mut capture) | Self::Return(mut capture) => {
                if capture.get_severity() > severity {
                    capture.set_severity(severity);
                }
                Self::Push(capture)
            }
            Self::Repeat => Self::Push(Capture::Empty),
        }
    }

    // Helper function, extracts a contained RefValue from the Accept.
    pub fn into_refvalue(self) -> RefValue {
        match self {
            Self::Push(capture) | Self::Return(capture) => capture.get_value(),
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
