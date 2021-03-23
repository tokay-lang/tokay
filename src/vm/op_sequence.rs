use super::*;
use crate::value::RefValue;

/** Sequence parser.

This parser collects a sequence of operations. According to these operation's
semantics, or when an entire sequence was completely recognized, the sequence
is getting accepted. Incomplete sequences are rejected, but might partly be
processed, including data changes, which is a wanted behavior.
*/

#[derive(Debug)]
pub struct Sequence {
    leftrec: bool,
    nullable: bool,
    items: Vec<(Op, Option<String>)>,
}

impl Sequence {
    pub fn new(items: Vec<(Op, Option<String>)>) -> Op {
        Self {
            leftrec: false,
            nullable: true,
            items,
        }
        .into_op()
    }
}

impl Runable for Sequence {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        // Empty sequence?
        if self.items.len() == 0 {
            return Ok(Accept::Next);
        }

        // Remember capturing positions
        let capture_start = context.runtime.stack.len();
        let reader_start = context.runtime.reader.tell();

        // Iterate over sequence
        for (item, alias) in &self.items {
            match item.run(context) {
                Err(Reject::Skip) => return Err(Reject::Skip),
                Err(reject) => {
                    context.runtime.stack.truncate(capture_start);
                    context.runtime.reader.reset(reader_start);
                    return Err(reject);
                }

                Ok(Accept::Next) => {
                    if let Some(alias) = alias {
                        context
                            .runtime
                            .stack
                            .push(Capture::Named(Box::new(Capture::Empty), alias.clone()))
                    } else {
                        context.runtime.stack.push(Capture::Empty)
                    }
                }

                Ok(Accept::Skip) => continue,

                Ok(Accept::Push(capture)) => {
                    if let Some(alias) = alias {
                        context
                            .runtime
                            .stack
                            .push(Capture::Named(Box::new(capture), alias.clone()))
                    } else {
                        context.runtime.stack.push(capture)
                    }
                }

                other => return other,
            }
        }

        /*
            When no explicit Return is performed, first try to collect any
            non-silent captures.
        */
        if let Some(capture) = context.collect(capture_start, false, true, 0) {
            Ok(Accept::Push(capture))
        } else {
            Ok(Accept::Next)
        }
    }

    fn finalize(
        &mut self,
        usages: &mut Vec<Vec<Op>>,
        statics: &Vec<RefValue>,
        leftrec: &mut bool,
        nullable: &mut bool,
        consumes: &mut bool,
    ) {
        /*
            Sequences are *the* special case for the transform
            facility. When a transform replaces one Op by
            multiple Ops, and this happens inside of a sequence,
            then the entire sequence must be extended in-place.

            So `a B c D e` may become `a x c y z e`.

            This could probably be made more fantastic with a
            real VM concept, but I'm just happy with this
            right now.
        */
        let mut end = self.items.len();
        let mut i = 0;

        while i < end {
            let item = self.items.get_mut(i).unwrap();

            if let Op::Usage(usage) = item.0 {
                let n = usages[usage].len();

                let old = self
                    .items
                    .splice(i..i + 1, usages[usage].drain(..).map(|item| (item, None)));

                // Re-assign alias-value of the lastly spliced item, if any.
                if let Some(alias) = old.into_iter().last().unwrap().1 {
                    self.items.get_mut(i + n - 1).unwrap().1 = Some(alias);
                }

                i += n;
                end = self.items.len();
            } else {
                i += 1
            }
        }

        // Finalize through children
        for (item, _) in self.items.iter_mut() {
            // While sequence is nullable, try to find left-recursions
            if self.nullable {
                item.finalize(
                    usages,
                    statics,
                    &mut self.leftrec,
                    &mut self.nullable,
                    consumes,
                );
            }
            // Otherwise, continue finalization and resolve usages only
            else {
                item.finalize(usages, &Vec::new(), &mut false, &mut true, consumes);
            }
        }

        *leftrec = self.leftrec;
        *nullable = self.nullable;
    }
}

impl std::fmt::Display for Sequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (item, _) in &self.items {
            write!(f, "{} ", item)?;
        }

        Ok(())
    }
}
