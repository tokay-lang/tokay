use super::*;
use crate::value::RefValue;

/** Block parser.

A block parser defines either an alternation of sequences or a grouped sequence
of VM instructions. The compiler has to guarantee for correct usage of the block
parser.
*/

#[derive(Debug)]
pub struct Block {
    items: Vec<Op>,
}

impl Block {
    pub fn new(items: Vec<Op>) -> Op {
        Self { items: items }.into_op()
    }
}

impl Runable for Block {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        let mut result = Ok(Accept::Next);
        let reader_start = context.runtime.reader.tell();

        for alt in self.items.iter() {
            result = alt.run(context);

            // Generally break on anything which is not Next.
            if !matches!(&result, Ok(Accept::Next) | Err(Reject::Next)) {
                // Push only accepts when input was consumed, otherwise the
                // push value is just discarded, except for the last item
                // being executed.
                if let Ok(Accept::Push(_)) = result {
                    // No consuming, no breaking!
                    if reader_start == context.runtime.reader.tell() {
                        continue;
                    }
                }

                break;
            }
        }

        result
    }

    fn resolve(&mut self, usages: &mut Vec<Vec<Op>>) {
        for alt in self.items.iter_mut() {
            alt.resolve(usages);
        }
    }

    fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<(bool, bool)> {
        let mut any_leftrec = false;
        let mut any_nullable = false;
        let mut consumes = false;

        for alt in self.items.iter_mut() {
            if let Some((leftrec, nullable)) = alt.finalize(statics, stack) {
                any_leftrec |= leftrec;
                any_nullable |= nullable;

                consumes = true;
            }
        }

        if consumes {
            Some((any_leftrec, any_nullable))
        } else {
            None
        }
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in &self.items {
            write!(f, "{}\n", item)?;
        }

        Ok(())
    }
}
