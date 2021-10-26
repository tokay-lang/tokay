use super::*;
use crate::error::Error;

/** Expecting construct.

This constructs expects its body to be accepted.
On failure, an error message is raised as Reject::Error.
*/

#[derive(Debug)]
pub struct Expect {
    body: ImlOp,
    msg: Option<String>,
}

impl Expect {
    pub fn new(body: ImlOp, msg: Option<String>) -> ImlOp {
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
                    Error::new(Some(start), format!("Expecting {}", self.body)).into_reject()
                }
            } else {
                Err(reject)
            }
        })
    }

    fn resolve(&mut self, usages: &mut Vec<Vec<ImlOp>>) {
        self.body.resolve(usages);
    }

    fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<Consumable> {
        self.body.finalize(statics, stack)
    }

    fn compile(&self, parselet: &ImlParselet) -> Vec<Op> {
        let body = self.body.compile(parselet);

        let mut ret = vec![Op::Frame(body.len() + 2)];

        ret.extend(body);

        ret.extend(vec![
            Op::Forward(2),
            Op::Error(Some(if let Some(msg) = &self.msg {
                msg.clone()
            } else {
                format!("Expecting {}", self.body)
            })),
            Op::Close,
        ]);

        ret
    }
}

impl std::fmt::Display for Expect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "expect {}", self.body)
    }
}
