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
    body: Op,
    min: usize,
    max: usize,
    silent: bool,
}

impl Repeat {
    pub fn new(body: Op, min: usize, max: usize, silent: bool) -> Op {
        assert!(max == 0 || max >= min);

        Self {
            body,
            min,
            max,
            silent,
        }
        .into_op()
    }

    pub fn kleene(body: Op) -> Op {
        Self::new(body, 0, 0, false)
    }

    pub fn positive(body: Op) -> Op {
        Self::new(body, 1, 0, false)
    }

    pub fn optional(body: Op) -> Op {
        Self::new(body, 0, 1, false)
    }

    pub fn kleene_silent(body: Op) -> Op {
        Self::new(body, 0, 0, true)
    }

    pub fn positive_silent(body: Op) -> Op {
        Self::new(body, 1, 0, true)
    }

    pub fn optional_silent(body: Op) -> Op {
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

    fn resolve(&mut self, usages: &mut Vec<Vec<Op>>) {
        self.body.resolve(usages);
    }

    fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<(bool, bool)> {
        if let Some((leftrec, nullable)) = self.body.finalize(statics, stack) {
            if self.min == 0 {
                Some((leftrec, true))
            } else {
                Some((leftrec, nullable))
            }
        } else {
            None
        }
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