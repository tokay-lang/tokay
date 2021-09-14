use super::*;
use crate::value::RefValue;

/** Sequence construct.

This intermediate language construct collects a sequence of operations or sequence of further
constructs.

According to these operation's semantics, or when an entire sequence is completely recognized,
the sequence is getting accepted. Incomplete sequences are rejected, but might partly be
processed, including data changes, which is a wanted behavior.
*/

#[derive(Debug)]
pub struct Sequence {
    pub(crate) consuming: Option<Consumable>, // Consumable state
    items: Vec<ImlOp>,
}

impl Sequence {
    pub fn new(items: Vec<ImlOp>) -> ImlOp {
        Self {
            consuming: None,
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
        for item in &self.items {
            match item.run(context) {
                Err(Reject::Skip) => return Err(Reject::Skip),
                Err(reject) => {
                    context.runtime.stack.truncate(capture_start);
                    context.runtime.reader.reset(reader_start);
                    return Err(reject);
                }

                Ok(Accept::Next) => context.runtime.stack.push(Capture::Empty),

                Ok(Accept::Hold) => continue,

                Ok(Accept::Push(capture)) => context.runtime.stack.push(capture),

                other => return other,
            }
        }

        /*
            When no explicit Return is performed, first try to collect any
            non-silent captures.
        */

        match context.collect(capture_start, false, true, true, 0) {
            Err(capture) => Ok(Accept::Push(capture)),
            Ok(Some(value)) => Ok(Accept::Push(Capture::Value(value, None, 5))),
            Ok(None) => Ok(Accept::Next),
        }
    }

    fn resolve(&mut self, usages: &mut Vec<Vec<ImlOp>>) {
        /*
            Sequences are *the* special case for symbol resolving.
            When a resolve replaces one Op by multiple Ops, and this
            happens inside of a sequence, then the entire sequence
            must be extended in-place.

            So `a B c D e` may become `a x c y z e`.

            This could probably be made more fantastic with ac real
            VM concept, but I'm just happy with this right now.
        */
        let mut end = self.items.len();
        let mut i = 0;

        while i < end {
            let item = self.items.get_mut(i).unwrap();

            if let ImlOp::Usage(usage) = *item {
                let n = usages[usage].len();

                self.items.splice(i..i + 1, usages[usage].drain(..));

                i += n;
                end = self.items.len();
            } else {
                i += 1
            }
        }

        for item in self.items.iter_mut() {
            item.resolve(usages);
        }
    }

    fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<Consumable> {
        let mut leftrec = false;
        let mut nullable = true;
        let mut consumes = false;

        for item in self.items.iter_mut() {
            if !nullable {
                break;
            }

            if let Some(consumable) = item.finalize(statics, stack) {
                leftrec |= consumable.leftrec;
                nullable = consumable.nullable;
                consumes = true;
            }
        }

        // Hold meta information about consuming state.
        if stack.len() == 1 && consumes {
            self.consuming = Some(Consumable { leftrec, nullable });
        }

        if consumes {
            Some(Consumable { leftrec, nullable })
        } else {
            None
        }
    }

    fn compile(&self, parselet: &ImlParselet) -> Vec<Op> {
        let mut ret = Vec::new();

        for item in self.items.iter() {
            ret.extend(item.compile(parselet));
        }

        if ret.len() > 1
            // Don't fuse anything which is directly stored.
            && !matches!(
                ret.last().unwrap(),
                Op::StoreGlobal(_) | Op::StoreFast(_) | Op::StoreFastCapture(_)
            )
        {
            ret.insert(0, Op::TryCapture(ret.len() + 2));
            ret.push(Op::Collect);
        }

        ret
    }
}

impl std::fmt::Display for Sequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for item in &self.items {
            write!(f, "{} ", item)?;
        }
        write!(f, ")")?;

        Ok(())
    }
}
