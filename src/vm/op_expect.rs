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
                if let Some(msg) = &self.msg {
                    Error::new(Some(context.runtime.reader.tell()), msg.clone()).into_reject()
                } else {
                    Error::new(
                        Some(context.runtime.reader.tell()),
                        format!("Expecting {}", self.body),
                    )
                    .into_reject()
                }
            } else {
                Err(reject)
            }
        })
    }

    fn finalize(
        &mut self,
        usages: &mut Vec<Vec<Op>>,
        statics: &Vec<RefValue>,
        leftrec: &mut bool,
        nullable: &mut bool,
        consumes: &mut bool,
    ) {
        self.body.replace_usage(usages);
        self.body
            .finalize(usages, statics, leftrec, nullable, consumes);
    }
}

impl std::fmt::Display for Expect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "expect {}", self.body)
    }
}
