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

        loop {
            match self.body.run(context) {
                Err(Reject::Next) => break,

                Err(reject) => {
                    context.runtime.stack.truncate(capture_start);
                    context.runtime.reader.reset(reader_start);
                    return Err(reject);
                }

                Ok(Accept::Next) => {}

                Ok(Accept::Push(capture)) => {
                    if !self.silent {
                        context.runtime.stack.push(capture)
                    }
                }

                Ok(accept) => return Ok(accept),
            }

            count += 1;

            if self.max > 0 && count == self.max {
                break;
            }
        }

        if count < self.min {
            context.runtime.stack.truncate(capture_start);
            context.runtime.reader.reset(reader_start);
            Err(Reject::Next)
        } else {
            // Push collected captures, if any
            if let Some(capture) = context.collect(capture_start, false, false, 1) {
                Ok(Accept::Push(capture))
            } else {
                // Otherwiese, push a capture of consumed range
                let range = context.runtime.reader.capture_from(&reader_start);
                if range.len() > 0 {
                    Ok(Accept::Push(Capture::Range(range, 0)))
                }
                // Else, just accept next
                else {
                    Ok(Accept::Next)
                }
            }
        }
    }

    fn finalize(
        &mut self,
        usages: &mut Vec<Vec<Op>>,
        statics: &Vec<RefValue>,
        leftrec: &mut bool,
        nullable: &mut bool,
    ) {
        self.body.replace_usage(usages);
        self.body.finalize(usages, statics, leftrec, nullable);

        if self.min == 0 {
            *nullable = true;
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
