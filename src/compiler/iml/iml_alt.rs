use super::*;
use crate::value::RefValue;

/** Alternation construct.

The alternation construct defines either an alternation of sequences or a grouped sequence
of instructions. An alternation is only performed when input is consumed, otherwise the
alternation works similar to a sequence of sequences.
*/

#[derive(Debug)]
pub struct Alternation {
    items: Vec<ImlOp>,
}

impl Alternation {
    pub fn new(items: Vec<ImlOp>) -> ImlOp {
        Self { items: items }.into_op()
    }
}

impl Runable for Alternation {
    fn resolve(&mut self, usages: &mut Vec<Vec<ImlOp>>) {
        for alt in self.items.iter_mut() {
            alt.resolve(usages);
        }
    }

    fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<Consumable> {
        let mut leftrec = false;
        let mut nullable = false;
        let mut consumes = false;

        for alt in self.items.iter_mut() {
            if let Some(consumable) = alt.finalize(statics, stack) {
                leftrec |= consumable.leftrec;
                nullable |= consumable.nullable;
                consumes = true;
            }
        }

        if consumes {
            Some(Consumable { leftrec, nullable })
        } else {
            None
        }
    }

    fn compile(&self, parselet: &ImlParselet) -> Vec<Op> {
        let mut ret = Vec::new();
        let mut iter = self.items.iter();
        let mut jumps = Vec::new();

        while let Some(item) = iter.next() {
            let alt = item.compile(parselet);

            if iter.len() > 0 {
                ret.push(Op::Fuse(alt.len() + 3));
                ret.extend(alt);
                ret.push(Op::Nop);
                ret.push(Op::Reset);

                jumps.push(ret.len() - 2);
            } else {
                ret.extend(alt);
            }
        }

        while let Some(addr) = jumps.pop() {
            ret[addr] = Op::ForwardIfConsumed(ret.len() - addr);
        }

        if self.items.len() > 1 {
            ret.insert(0, Op::Frame(0));
            ret.push(Op::Close);
        }

        ret
    }
}

impl std::fmt::Display for Alternation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for item in &self.items {
            write!(f, "{} ", item)?;
        }
        write!(f, "}}")?;

        Ok(())
    }
}
