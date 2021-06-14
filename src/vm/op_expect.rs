use super::*;
use crate::error::Error;

/** Expecting parser.

This parser expects its sub-parser to be accepted.
On error, a helpul error message is raised as Reject::Error.
*/

#[derive(Debug)]
pub struct Expect {
    body: Op,
    msg: Option<String>,
}

impl Expect {
    pub fn new(body: Op, msg: Option<String>) -> Op {
        Self { body, msg }.into_op()
    }
}

impl Runable for Expect {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        self.body.run(context).or_else(|reject| {
            if let Reject::Next = reject {
                let start = context.runtime.reader.tell();

                if let Some(msg) = &self.msg {
                    Error::new(Some(start), msg.clone()).into_reject()
                } else {
                    Error::new(
                        Some(start),
                        format!("Expecting {}", self.body),
                    )
                    .into_reject()
                }
            } else {
                Err(reject)
            }
        })
    }

    fn resolve(&mut self, usages: &mut Vec<Vec<Op>>) {
        self.body.resolve(usages);
    }

    fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<(bool, bool)> {
        self.body.finalize(statics, stack)
    }
}

impl std::fmt::Display for Expect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "expect {}", self.body)
    }
}
