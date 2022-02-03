use crate::reader::{Range, Reader};
use crate::value::{RefValue, Value};

/// Captures are stack items where the VM operates on.
#[derive(Debug, Clone)]
pub enum Capture {
    Empty,                               // Empty capture
    Range(Range, Option<String>, u8),    // Captured range
    Value(RefValue, Option<String>, u8), // Captured value
}

impl Capture {
    /** Extracts a value from a capture.

    In case the capture is a range, the range is extracted as a string from the reader. */
    pub(super) fn extract(&mut self, reader: &Reader) -> RefValue {
        match self {
            Capture::Empty => Value::Void.into(),
            Capture::Range(range, alias, severity) => {
                let value = RefValue::from(reader.extract(range));
                *self = Capture::Value(value.clone(), alias.clone(), *severity);
                value
            }
            Capture::Value(value, ..) => value.clone(),
        }
    }

    pub fn get_value(&self) -> RefValue {
        match self {
            Capture::Empty => Value::Void.into(),
            Capture::Range(..) => {
                panic!("Cannot retrieve value of Capture::Range, use self.extract() first!")
            }
            Capture::Value(value, ..) => value.clone(),
        }
    }

    pub fn get_severity(&self) -> u8 {
        match self {
            Capture::Range(_, _, severity) | Capture::Value(_, _, severity) => *severity,
            _ => 0,
        }
    }

    pub fn set_severity(&mut self, new_severity: u8) {
        match self {
            Capture::Range(_, _, severity) | Capture::Value(_, _, severity) => {
                *severity = new_severity
            }
            _ => {}
        }
    }

    // Degrades a capture to a severity to a capture with zero severity.
    // This is done when a capture is read.
    pub fn degrade(&mut self) {
        match self {
            Capture::Range(_, _, severity) | Capture::Value(_, _, severity) if *severity <= 5 => {
                *severity = 0;
            }
            _ => {}
        }
    }
}

impl From<Value> for Capture {
    fn from(value: Value) -> Self {
        Capture::Value(value.into(), None, 10)
    }
}

impl From<RefValue> for Capture {
    fn from(value: RefValue) -> Self {
        Capture::Value(value, None, 10)
    }
}
