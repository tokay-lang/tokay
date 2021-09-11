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
    silent: bool,
}

impl Repeat {
    pub fn new(body: ImlOp, min: usize, max: usize, silent: bool) -> ImlOp {
        assert!(max == 0 || max >= min);

        Self {
            body,
            min,
            max,
            silent,
        }
        .into_op()
    }

    pub fn kleene(body: ImlOp) -> ImlOp {
        Self::new(body, 0, 0, false)
    }

    pub fn positive(body: ImlOp) -> ImlOp {
        Self::new(body, 1, 0, false)
    }

    pub fn optional(body: ImlOp) -> ImlOp {
        Self::new(body, 0, 1, false)
    }

    pub fn kleene_silent(body: ImlOp) -> ImlOp {
        Self::new(body, 0, 0, true)
    }

    pub fn positive_silent(body: ImlOp) -> ImlOp {
        Self::new(body, 1, 0, true)
    }

    pub fn optional_silent(body: ImlOp) -> ImlOp {
        Self::new(body, 0, 1, true)
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

                    if !self.silent {
                        context.runtime.stack.push(capture)
                    }
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
        } else if self.silent {
            Ok(Accept::Next)
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
            (0, 0) | (1, 0) => {
                let start = if self.min == 1 {
                    let copy = ret.clone();

                    ret.push(Op::Consumed);
                    ret.push(Op::Nop); // placeholder for Op::ForwardIfFalse
                    let start = ret.len();

                    ret.extend(copy);
                    start
                } else {
                    0
                };

                // Kleene
                ret.push(Op::Consumed);
                ret.push(Op::ForwardIfFalse(2));
                ret.push(Op::Backward(ret.len() - start));
                ret.push(Op::Collect);
                ret.insert(0, Op::Capture(ret.len() + 1));

                if start > 0 {
                    ret[start] = Op::ForwardIfFalse(ret.len() - start);
                }
            }
            (0, 1) => {
                // Optional
                ret.push(Op::Collect);
                ret.insert(0, Op::Capture(ret.len() + 1));
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
