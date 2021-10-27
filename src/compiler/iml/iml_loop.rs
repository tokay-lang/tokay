use super::*;

/** Loop-construct */

#[derive(Debug)]
pub struct Loop {
    body: ImlOp,
}

impl Loop {
    pub fn new(body: ImlOp) -> ImlOp {
        Self { body }.into_op()
    }
}

impl Runable for Loop {
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
        let mut ret = Vec::new();

        ret.push(Op::Frame(0));

        // compile body and fix wildcard forward- and backward-calls.
        let body = self.body.compile(parselet);
        let body_len = body.len();

        for (i, op) in body.into_iter().enumerate() {
            ret.push(match op {
                Op::Forward(0) => Op::Forward(body_len - i - 1),
                Op::Backward(0) => Op::Backward(i - 1),
                op => op,
            })
        }

        ret.push(Op::Close);

        ret
    }
}

impl std::fmt::Display for Loop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "loop {}", self.body)
    }
}
