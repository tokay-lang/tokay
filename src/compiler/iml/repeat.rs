use super::*;

/** Repeating parser.

This is a simple programmatic sequential repetition. For several reasons,
repetitions can also be expressed on a specialized token-level or by the grammar
itself using left- and right-recursive structures, resulting in left- or right-
leaning parse trees.
*/

#[derive(Debug)]
pub struct ImlRepeat {
    body: ImlOp,
    min: usize,
    max: usize,
}

impl ImlRepeat {
    pub fn new(body: ImlOp, min: usize, max: usize) -> ImlOp {
        assert!(max == 0 || max >= min);

        Self { body, min, max }.into_op()
    }

    pub fn kleene(body: ImlOp) -> ImlOp {
        Self::new(body, 0, 0)
    }

    pub fn positive(body: ImlOp) -> ImlOp {
        Self::new(body, 1, 0)
    }

    pub fn optional(body: ImlOp) -> ImlOp {
        Self::new(body, 0, 1)
    }
}

impl Compileable for ImlRepeat {
    fn resolve(&mut self, usages: &mut Vec<Vec<ImlOp>>) {
        self.body.resolve(usages);
    }

    fn finalize(
        &mut self,
        values: &Vec<ImlValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<Consumable> {
        if let Some(consumable) = self.body.finalize(values, stack) {
            if self.min == 0 {
                Some(Consumable {
                    leftrec: consumable.leftrec,
                    nullable: true,
                })
            } else {
                Some(consumable)
            }
        } else {
            None
        }
    }

    fn compile(&self, parselet: &ImlParselet) -> Vec<Op> {
        let body = self.body.compile(parselet);
        let body_len = body.len();

        let mut ret = Vec::new();

        match (self.min, self.max) {
            (0, 0) => {
                // Kleene
                ret.extend(vec![
                    Op::Frame(0),            // The overall capture
                    Op::Frame(body_len + 5), // The fused capture for repetition
                ]);
                ret.extend(body); // here comes the body
                ret.extend(vec![
                    Op::ForwardIfConsumed(2), // When consumed we can commit and jump backward
                    Op::Forward(3),           // otherwise leave the loop
                    Op::Commit,
                    Op::Backward(body_len + 3), // repeat the body
                    Op::Close,
                    Op::Collect(1), // collect only values with severity > 0
                    Op::Close,
                ]);
            }
            (1, 0) => {
                // Positive
                ret.push(Op::Frame(0)); // The overall capture
                ret.extend(body.clone()); // here comes the body for the first time
                ret.extend(vec![
                    Op::ForwardIfConsumed(2), // ImlIf nothing was consumed, then...
                    Op::Next,                 //...reject
                    Op::Frame(body_len + 5),  // The fused capture for repetition
                ]);
                ret.extend(body); // here comes the body again inside the repetition
                ret.extend(vec![
                    Op::ForwardIfConsumed(2), // When consumed we can commit and jump backward
                    Op::Forward(3),           // otherwise leave the loop
                    Op::Commit,
                    Op::Backward(body_len + 3), // repeat the body
                    Op::Close,
                    Op::Collect(1), // collect only values with severity > 0
                    Op::Close,
                ]);
            }
            (0, 1) => {
                // Optional
                ret.push(Op::Frame(body_len + 2));
                ret.extend(body);
                ret.push(Op::Collect(1)); // collect only values with severity > 0
                ret.push(Op::Close);
            }
            (1, 1) => {}
            (_, _) => unimplemented!(
                "ImlRepeat construct with min/max configuration > 1 not implemented yet"
            ),
        };

        ret
    }
}

impl std::fmt::Display for ImlRepeat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.min, self.max) {
            (0, 1) => write!(f, "opt {}", self.body),
            (0, _) => write!(f, "kle {}", self.body),
            (1, _) => write!(f, "pos {}", self.body),
            (m, n) => write!(f, "{}{{{}, {}}}", self.body, m, n), // todo syntax unclear for this...
        }
    }
}
