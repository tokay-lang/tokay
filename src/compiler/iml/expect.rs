use super::*;

/** Expecting construct.

This constructs expects its body to be accepted.
On failure, an error message is raised as Reject::Error.
*/

#[derive(Debug)]
pub struct ImlExpect {
    body: ImlOp,
    msg: Option<String>,
}

impl ImlExpect {
    pub fn new(body: ImlOp, msg: Option<String>) -> ImlOp {
        Self { body, msg }.into_op()
    }
}

impl Compileable for ImlExpect {
    fn finalize(
        &mut self,
        values: &Vec<ImlValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<Consumable> {
        self.body.finalize(values, stack)
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

impl std::fmt::Display for ImlExpect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "expect {}", self.body)
    }
}
