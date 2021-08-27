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

impl Runable for Not {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        let res = match self.body.run(context) {
            Ok(_) => Err(Reject::Next),
            Err(Reject::Error(err)) => Err(Reject::Error(err)),
            Err(_) => Ok(Accept::Next),
        };

        res
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
        let mut ret = Vec::new();

        ret.push(Op::Nop); // Frame placeholder
        ret.extend(self.body.compile(parselet));
        ret.push(Op::Invert);

        ret[0] = Op::Segment(ret.len());
        ret
    }
}

impl std::fmt::Display for Not {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not {}", self.body)
    }
}
