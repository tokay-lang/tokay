use super::*;

/** Peeking parser.

This parser runs its sub-parser and returns its result, but resets the reading-context
afterwards. It can be used to look ahead parsing constructs, but leaving the rest of the
parser back to its original position, to decide.

Due to Tokays memoizing features, the parsing will only be done once, and is remembered.
*/

#[derive(Debug)]
pub struct Peek {
    body: ImlOp,
}

impl Peek {
    pub fn new(body: ImlOp) -> ImlOp {
        Self { body }.into_op()
    }
}

impl Runable for Peek {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        let reader_start = context.runtime.reader.tell();
        let ret = self.body.run(context);
        context.runtime.reader.reset(reader_start);
        ret
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
        ret.push(Op::Discard);

        ret[0] = Op::Segment(ret.len());
        ret
    }
}

impl std::fmt::Display for Peek {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "peek {}", self.body)
    }
}
