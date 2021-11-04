use super::*;

/** Negate parser.

This parser runs its sub-parser and returns its negated result, so that an accept becomes
rejected and vice-versa.
*/

#[derive(Debug)]
pub struct Not {
    body: ImlOp,
}

impl Not {
    pub fn new(body: ImlOp) -> ImlOp {
        Self { body }.into_op()
    }
}

impl Compileable for Not {
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

        let body = self.body.compile(parselet);

        ret.push(Op::Frame(body.len() + 3));
        ret.extend(body);
        ret.push(Op::Close);
        ret.push(Op::Next);

        ret
    }
}

impl std::fmt::Display for Not {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not {}", self.body)
    }
}
