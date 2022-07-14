use super::*;

/** Peek parser.

This parser runs its sub-parser and returns its result, but resets the reading-context
afterwards. It can be used to look ahead parsing constructs, but leaving the rest of the
parser back to its original position, to decide.

Due to Tokays memoizing features, the parsing will only be done once, and is remembered.
*/

#[derive(Debug)]
pub struct ImlPeek {
    body: ImlOp,
}

impl ImlPeek {
    pub fn new(body: ImlOp) -> ImlOp {
        Self { body }.into_op()
    }
}

impl Compileable for ImlPeek {
    fn finalize(
        &mut self,
        values: &Vec<ImlValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<Consumable> {
        self.body.finalize(values, stack)
    }

    fn compile(&self, parselet: &ImlParselet) -> Vec<Op> {
        let mut ret = Vec::new();

        ret.push(Op::Frame(0));
        ret.extend(self.body.compile(parselet));
        ret.push(Op::Reset);
        ret.push(Op::Close);

        ret
    }
}

impl std::fmt::Display for ImlPeek {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "peek {}", self.body)
    }
}
