use crate::reader::{Range, Reader};
use crate::value::RefValue;

/** Captures are stack items where the VM operates on.

A capture can either be just empty, a range from the input or a full qualified value (RefValue).
In case the capture is a range, it can be turned into a string value on demand and in-place.
*/
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
            Capture::Empty => crate::value!(void),
            Capture::Range(range, alias, severity) => {
                let value = RefValue::from(reader.get(range));
                *self = Capture::Value(value.clone(), alias.clone(), *severity);
                value
            }
            Capture::Value(value, ..) => value.clone(),
        }
    }

    pub fn get_value(&self) -> RefValue {
        match self {
            Capture::Empty => crate::value!(void),
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

impl From<RefValue> for Capture {
    fn from(value: RefValue) -> Self {
        Capture::Value(value, None, 10)
    }
}

#[test]
// Testing sequence captures
fn test_captures() {
    assert_eq!(
        crate::run("'a' 'b' $1 * 2 + $2 * 3", "ab"),
        Ok(Some(crate::value!("aabbb")))
    );

    assert_eq!(
        crate::run("a=2 'a' 'b' $(a + 1) * 3+ $(a) * 2", "ab"),
        Ok(Some(crate::value!("bbbaa")))
    );

    assert_eq!(
        crate::run("'a' $0 = \"yes\" 'b'+", "abbb"),
        Ok(Some(crate::value!("yes")))
    );
}
