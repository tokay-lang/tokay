use super::*;

/** Loop-construct */

#[derive(Debug)]
pub struct Loop {
    init: ImlOp,
    condition: ImlOp,
    body: ImlOp,
}

impl Loop {
    pub fn new(init: ImlOp, condition: ImlOp, body: ImlOp) -> ImlOp {
        Self {
            init,
            condition,
            body,
        }
        .into_op()
    }
}

impl Runable for Loop {
    fn resolve(&mut self, usages: &mut Vec<Vec<ImlOp>>) {
        self.init.resolve(usages);
        self.condition.resolve(usages);
        self.body.resolve(usages);
    }

    fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<Consumable> {
        self.init.finalize(statics, stack); // todo: incomplete as result is thrown away.
        self.condition.finalize(statics, stack); // todo: incomplete as result is thrown away.
        self.body.finalize(statics, stack)
    }

    fn compile(&self, parselet: &ImlParselet) -> Vec<Op> {
        let mut ret = Vec::new();

        ret.extend(self.init.compile(parselet));

        let mut body = self.condition.compile(parselet);
        if !body.is_empty() {
            body.push(Op::ForwardIfTrue(2));
            body.push(Op::Break);
        }

        body.extend(self.body.compile(parselet));

        ret.push(Op::Loop(body.len() + 2));
        ret.extend(body);
        ret.push(Op::Continue);

        ret
    }
}

impl std::fmt::Display for Loop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "loop {}", self.body)
    }
}
