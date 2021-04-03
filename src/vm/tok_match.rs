use super::*;
use crate::reader::Reader;

/** Match scanner.

This scanner implements the recognition of an exact character sequence within
the input stream.
*/

#[derive(Debug)]
pub struct Match {
    string: String,
    silent: bool,
}

impl Match {
    fn _new(string: &str, silent: bool) -> Op {
        Self {
            string: string.to_string(),
            silent,
        }
        .into_op()
    }

    pub fn new(string: &str) -> Op {
        Match::_new(string, false)
    }

    pub fn new_silent(string: &str) -> Op {
        Match::_new(string, true)
    }

    // Todo: Match Until!
}

impl Token for Match {
    fn read(&self, reader: &mut Reader) -> Result<Accept, Reject> {
        let start = reader.tell();

        for ch in self.string.chars() {
            if let Some(c) = reader.next() {
                if c != ch {
                    // fixme: Optimize me!
                    reader.reset(start);
                    return Err(Reject::Next);
                }
            } else {
                // fixme: Optimize me!
                reader.reset(start);
                return Err(Reject::Next);
            }
        }

        let range = reader.capture_last(self.string.len());

        Ok(Accept::Push(if self.silent {
            Capture::Range(range, 0)
        } else {
            Capture::Range(range, 5)
        }))
    }
}

impl std::fmt::Display for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.silent {
            write!(f, "'{}'", self.string)
        } else {
            write!(f, "''{}''", self.string)
        }
    }
}
