//! Tokay virtual machine

mod capture;
mod context;
mod op;
mod program;
mod runtime;

pub use capture::*;
pub use context::*;
pub use op::*;
pub use program::*;
pub use runtime::*;

use crate::error::Error;
use crate::value::{RefValue, Value};

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

impl From<Value> for Result<Accept, Reject> {
    fn from(value: Value) -> Self {
        Ok(Accept::Push(value.into()))
    }
}

// --- Reject ------------------------------------------------------------------

/// Representing the Err-value result on a branched run of the VM.
#[derive(Debug, Clone)]
pub enum Reject {
    Next,   // soft-reject, skip to next sequence
    Skip,   // hard-reject, silently drop current parselet
    Return, // hard-reject current parselet ('return'/'reject'-keyword)
    Main,   // hard-reject current parselet and exit to main scope ('escape'-keyword)
    Error(Box<Error>), //hard-reject with error message (runtime error)
            // todo: Exit(u32) // stop entire program with exit code
}

impl From<Error> for Reject {
    fn from(error: Error) -> Self {
        Reject::Error(Box::new(error))
    }
}
