use super::*;

/** Alternation construct.

The alternation construct defines either an alternation of sequences or a grouped sequence
of instructions. An alternation is only performed when input is consumed, otherwise the
alternation works similar to a sequence of sequences.
*/

#[derive(Debug)]
pub struct ImlAlternation {
    items: Vec<ImlOp>,
}

impl ImlAlternation {
    pub fn new(items: Vec<ImlOp>) -> ImlOp {
        Self { items: items }.into_op()
    }
}

impl Compileable for ImlAlternation {
    fn resolve(&mut self, usages: &mut Vec<Vec<ImlOp>>) {
        for alt in self.items.iter_mut() {
            alt.resolve(usages);
        }
    }

    fn finalize(
        &mut self,
        values: &Vec<ImlValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<Consumable> {
        let mut leftrec = false;
        let mut nullable = false;
        let mut consumes = false;

        for alt in self.items.iter_mut() {
            if let Some(consumable) = alt.finalize(values, stack) {
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

impl std::fmt::Display for ImlAlternation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for item in &self.items {
            write!(f, "{} ", item)?;
        }
        write!(f, "}}")?;

        Ok(())
    }
}
