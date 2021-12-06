//! Implementation of an error object that can occur during Tokay's program compilation or execution
use linkme::distributed_slice;

use crate::builtin::{Builtin, BUILTINS};
use crate::reader::Offset;
use crate::value::Value;
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

    /// Turn an error into a string
    pub fn into_string(self) -> String {
        format!("{}", self)
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

#[distributed_slice(BUILTINS)]
static ERROR: Builtin = Builtin {
    name: "error",
    signature: "msg ? collect",
    func: |context, args| {
        let msg = args[0].as_ref().unwrap();
        let collect = args[1]
            .as_ref()
            .map_or(false, |value| value.borrow().is_true());

        let mut msg = msg.borrow().to_string();

        if collect {
            if let Ok(Some(value)) = context.collect(context.capture_start, false, true, false, 0) {
                let value = value.borrow();

                if let Value::String(s) = &*value {
                    msg.push_str(&format!(": '{}'", s))
                } else {
                    msg.push_str(&format!(": {}", value.repr()))
                }
            }
        }

        Error::new(Some(context.runtime.reader.tell()), msg).into_reject()
    },
};
