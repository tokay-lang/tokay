use super::*;

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
                let pos = context.runtime.reader.tell();

                if let Some(msg) = &self.msg {
                    Err(Reject::Error(format!(
                        "Line {}, column {}: {}",
                        pos.row, pos.col, msg
                    )))
                } else {
                    Err(Reject::Error(format!(
                        "Line {}, column {}: Expecting {}",
                        pos.row, pos.col, self.body
                    )))
                }
            } else {
                Err(reject)
            }
        })
    }

    fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        usages: &mut Vec<Vec<Op>>,
        leftrec: &mut bool,
        nullable: &mut bool,
    ) {
        self.body.replace_usage(usages);
        self.body.finalize(statics, usages, leftrec, nullable);
    }
}

impl std::fmt::Display for Expect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "expect {}", self.body)
    }
}
