use super::*;
use crate::value::RefValue;

/** Repeating parser.

This is a simple programmatic sequential repetition. For several reasons,
repetitions can also be expressed on a specialized token-level or by the grammar
itself using left- and right-recursive structures, resulting in left- or right-
leaning parse trees.
*/

#[derive(Debug)]
pub struct Repeat {
    body: ImlOp,
    min: usize,
    max: usize,
}

impl Repeat {
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

impl Runable for Repeat {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        // Remember capturing positions
        let capture_start = context.runtime.stack.len();
        let reader_start = context.runtime.reader.tell();

        let mut count: usize = 0;

        /*
        let debug = match &context.parselet.name {
            Some(name) if name == "Block" => true,
            _ => false
        };

        if debug {
            println!("--- {:?} ---\n{:#?}", context.parselet.name, self);
        }
        */
        loop {
            let loop_start = context.runtime.reader.tell();

            match self.body.run(context) {
                Err(Reject::Next) => break,

                Err(reject) => {
                    context.runtime.stack.truncate(capture_start);
                    context.runtime.reader.reset(reader_start);
                    return Err(reject);
                }

                Ok(Accept::Next) => {}

                Ok(Accept::Push(capture)) => {
                    count += 1;
                    context.runtime.stack.push(capture)
                }

                Ok(accept) => return Ok(accept),
            }

            if (self.max > 0 && count == self.max) || loop_start == context.runtime.reader.tell() {
                break;
            }
        }

        if count < self.min {
            context.runtime.stack.truncate(capture_start);
            context.runtime.reader.reset(reader_start);
            Err(Reject::Next)
        } else {
            match context.collect(capture_start, false, true, true, 1) {
                Err(capture) => Ok(Accept::Push(capture)),
                Ok(Some(value)) => Ok(Accept::Push(Capture::Value(value, None, 5))),
                Ok(None) => {
                    /*
                    // Push a capture of consumed range with no severity
                    let range = context.runtime.reader.capture_from(&reader_start);
                    if range.len() > 0 {
                        Ok(Accept::Push(Capture::Range(range, None, 0)))
                    }
                    */
                    Ok(Accept::Next)
                }
            }
        }
    }

    fn resolve(&mut self, usages: &mut Vec<Vec<ImlOp>>) {
        self.body.resolve(usages);
    }

    fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<Consumable> {
        if let Some(consumable) = self.body.finalize(statics, stack) {
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
        let mut ret = self.body.compile(parselet);

        match (self.min, self.max) {
            (0, 0) => {
                ret.insert(0, Op::FusedCapture(ret.len() + 3));
                ret.push(Op::Commit);
                ret.push(Op::Backward(ret.len()));

                // Surround the result of the repetition by additional frame
                ret.insert(0, Op::Capture(0));
                ret.push(Op::Collect);
                ret.push(Op::Commit);
            }
            (1, 0) => {
                // Positive

                // First of all, create a copy of the body for repetition.
                let mut repeat = ret.clone();

                repeat.insert(0, Op::FusedCapture(repeat.len() + 3));
                repeat.push(Op::Commit);
                repeat.push(Op::Backward(repeat.len()));

                // Patch possible FusedCapture to end of entire block.
                if let Op::FusedCapture(goto) = ret.first_mut().unwrap() {
                    *goto += repeat.len() + 3;
                }

                ret.extend(repeat);

                // Surround the result of the repetition by additional frame
                ret.insert(0, Op::FusedCapture(ret.len() + 3));
                ret.push(Op::Collect);
                ret.push(Op::Commit);
            }
            (0, 1) => {
                // Optional
                ret.insert(0, Op::Capture(ret.len() + 1));
                ret.push(Op::Commit); // fixme: The recursive implementation only collects >0 severity here!
            }
            (1, 1) => {}
            (_, _) => unimplemented!(
                "Repeat construct with min/max configuration > 1 not implemented yet"
            ),
        };

        ret
    }
}

impl std::fmt::Display for Repeat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.min, self.max) {
            (0, 1) => write!(f, "opt {}", self.body),
            (0, _) => write!(f, "kle {}", self.body),
            (1, _) => write!(f, "pos {}", self.body),
            (m, n) => write!(f, "{}{{{}, {}}}", self.body, m, n), // todo syntax unclear for this...
        }
    }
}
