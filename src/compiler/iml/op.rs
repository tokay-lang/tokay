/*! Intermediate code representation. */
use super::*;

/*
    Todo / Ideas for this module

    - [ ] Usage is integrated into ImlOp?
      - eventually by using Rc<RefCell>?
    - [ ] Compilable is integrated into ImlOp as full variation
      - [x] Alternation, Sequence, If, Loop
      - [ ] Replace expect, not, peek, repeat by their generic counterparts
        - [ ] Thinking about inline-parselets, whose VM code will be inserted right in place (or already on ImlOp level)
    - [ ] Finalization must be re-defined, as this is only possible on consumable constructs
      - find left-recursions
      - find nullables
*/

#[derive(Debug)]
pub enum ImlOp {
    Nop,                               // Empty operation
    Op(Op),                            // VM Operation
    Usage(usize),                      // (yet) unresolved usage
    Compileable(Box<dyn Compileable>), // Compileable item (DEPRECATED: Remaining implementors will be replaced by generic parselets)
    // Alternation (Block) of sequences or ops
    Alt {
        alts: Vec<ImlOp>,
    },
    // Sequence of ops, optionally framed
    Seq {
        seq: Vec<ImlOp>,
        framed: bool, /* According to these operation's semantics, or when an entire sequence is completely recognized,
                      the sequence is getting accepted. Incomplete sequences are rejected, but might partly be
                      processed, including data changes, which is a wanted behavior. */
    },
    // Conditional block
    If {
        peek: bool,       // Peek test value instead of pop (required to implement the or-operator)
        test: bool,       // Boolean value to test against (true or false)
        then: Box<ImlOp>, // Conditional code path
        else_: Box<ImlOp>, // Optional code path executed otherwise
    },
    // Loop construct
    Loop {
        consuming: Option<Consumable>, // Consumable state: FIXME: Remove and replace asap.
        init: Box<ImlOp>,              // Initial operation
        condition: Box<ImlOp>,         // Abort condition
        body: Box<ImlOp>,              // Iterating body
    },
}

impl ImlOp {
    pub fn from_vec(ops: Vec<ImlOp>) -> Self {
        match ops.len() {
            0 => ImlOp::Nop,
            1 => ops.into_iter().next().unwrap(),
            _ => ImlOp::Seq {
                seq: ops,
                framed: false,
            },
        }
    }

    pub fn into_kleene(self) -> Self {
        ImlRepeat::kleene(self)
    }

    pub fn into_positive(self) -> Self {
        ImlRepeat::positive(self)
    }

    pub fn into_optional(self) -> Self {
        ImlRepeat::optional(self)
    }
}

impl Compileable for ImlOp {
    fn compile(&self, parselet: &ImlParselet) -> Vec<Op> {
        match self {
            ImlOp::Nop => Vec::new(),
            ImlOp::Op(op) => vec![op.clone()],
            ImlOp::Usage(_) => panic!("Cannot compile ImlOp::Usage"),
            ImlOp::Compileable(compileable) => compileable.compile(parselet),
            ImlOp::Alt { alts } => {
                let mut ret = Vec::new();
                let mut iter = alts.iter();
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

                if alts.len() > 1 {
                    ret.insert(0, Op::Frame(0));
                    ret.push(Op::Close);
                }

                ret
            }
            ImlOp::Seq { seq, framed } => {
                let mut ret = Vec::new();

                for item in seq.iter() {
                    let mut ops = item.compile(parselet);

                    // In case there is an inline operation within a sequence, its result must be duplicated
                    // to stay consistent inside of the sequence's result.
                    if *framed {
                        match ops.last().unwrap() {
                            Op::UnaryOp(op) | Op::BinaryOp(op) if op.starts_with("i") => {
                                ops.push(Op::Sep);
                            }
                            _ => {}
                        }
                    }

                    ret.extend(ops);
                }

                // Create a frame and collect in framed mode and when there's more than one operation inside ret.
                if *framed
                    && ret
                        .iter()
                        .map(|op| if matches!(op, Op::Offset(_)) { 0 } else { 1 })
                        .sum::<usize>()
                        > 1
                {
                    ret.insert(0, Op::Frame(0));
                    ret.push(Op::Collect(0, 5));
                    ret.push(Op::Close);
                }

                ret
            }
            ImlOp::If {
                peek,
                test,
                then,
                else_,
            } => {
                let mut ret = Vec::new();

                // Clone on peek
                if *peek {
                    ret.push(Op::Clone);
                }

                // Then-part
                let then = then.compile(parselet);

                let backpatch = ret.len();
                ret.push(Op::Nop); // Backpatch operation placeholder

                if *peek {
                    ret.push(Op::Drop)
                }

                let mut jump = then.len() + 1;
                ret.extend(then);

                if !*peek {
                    // Else-part
                    let else_ = else_.compile(parselet);

                    if !else_.is_empty() {
                        ret.push(Op::Forward(else_.len() + 1));
                        jump += 1;
                        ret.extend(else_);
                    }
                } else {
                    jump += 1;
                }

                // Insert the final condition and its failure target.
                if *test {
                    ret[backpatch] = Op::ForwardIfFalse(jump);
                } else {
                    ret[backpatch] = Op::ForwardIfTrue(jump);
                }

                ret
            }
            ImlOp::Loop {
                consuming,
                init,
                condition,
                body,
            } => {
                let mut ret = Vec::new();

                ret.extend(init.compile(parselet));

                let mut repeat = condition.compile(parselet);
                if !repeat.is_empty() {
                    repeat.push(Op::ForwardIfTrue(2));
                    repeat.push(Op::Break);
                }

                repeat.extend(body.compile(parselet));

                ret.push(Op::Loop(
                    repeat.len() + if consuming.is_some() { 3 } else { 2 },
                ));
                if consuming.is_some() {
                    ret.push(Op::Fuse(repeat.len() + 2));
                }

                ret.extend(repeat);
                ret.push(Op::Continue);

                if consuming.is_some() {
                    ret.push(Op::Break);
                }

                ret
            }
        }
    }

    fn resolve(&mut self, usages: &mut Vec<Vec<ImlOp>>) {
        match self {
            ImlOp::Usage(usage) => *self = Self::from_vec(usages[*usage].drain(..).collect()),
            ImlOp::Compileable(compileable) => compileable.resolve(usages),
            ImlOp::Alt { alts } => {
                for alt in alts {
                    alt.resolve(usages);
                }
            }
            ImlOp::Seq { seq, .. } => {
                for item in seq.iter_mut() {
                    item.resolve(usages);
                }
            }
            ImlOp::If { then, else_, .. } => {
                then.resolve(usages);
                else_.resolve(usages);
            }
            ImlOp::Loop {
                init,
                condition,
                body,
                ..
            } => {
                init.resolve(usages);
                condition.resolve(usages);
                body.resolve(usages);
            }
            _ => {}
        }
    }

    fn finalize(
        &mut self,
        values: &Vec<ImlValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<Consumable> {
        match self {
            ImlOp::Op(Op::CallStatic(target)) => {
                match &values[*target] {
                    ImlValue::Parselet(parselet) => {
                        if stack.len() > 0 {
                            if let Ok(mut parselet) = parselet.try_borrow_mut() {
                                if parselet.consuming.is_none() {
                                    return None;
                                }

                                stack.push((
                                    *target,
                                    if let Some(consuming) = parselet.consuming.as_ref() {
                                        consuming.nullable
                                    } else {
                                        false
                                    },
                                ));
                                let ret = parselet.finalize(values, stack);
                                stack.pop();

                                // --- Incomplete solution for the problem described in test/testindirectleftrec ---
                                // If left-recursion detected and called parselet is already
                                // left-recursive, thread currently analyzed parselet as
                                // not left-recursive here!
                                /*
                                if ret.0 && parselet.leftrec {
                                    ret.0 = false;
                                }
                                */

                                ret
                            } else {
                                for i in 0..stack.len() {
                                    if *target == stack[i].0 {
                                        return Some(Consumable {
                                            leftrec: i == 0,
                                            nullable: stack[i].1,
                                        });
                                    }
                                }

                                panic!("Can't find entry for {}", *target)
                            }
                        } else {
                            None
                        }
                    }

                    object => {
                        if object.is_consuming() {
                            Some(Consumable {
                                leftrec: false,
                                nullable: object.is_nullable(),
                            })
                        } else {
                            None
                        }
                    }
                }
            }
            ImlOp::Compileable(runable) => runable.finalize(values, stack),
            ImlOp::Alt { alts } => {
                let mut leftrec = false;
                let mut nullable = false;
                let mut consumes = false;

                for alt in alts.iter_mut() {
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
            ImlOp::Seq { seq, .. } => {
                let mut leftrec = false;
                let mut nullable = true;
                let mut consumes = false;

                for item in seq.iter_mut() {
                    if !nullable {
                        break;
                    }

                    if let Some(consumable) = item.finalize(values, stack) {
                        leftrec |= consumable.leftrec;
                        nullable = consumable.nullable;
                        consumes = true;
                    }
                }

                if consumes {
                    Some(Consumable { leftrec, nullable })
                } else {
                    None
                }
            }
            ImlOp::If { then, else_, .. } => {
                let then = then.finalize(values, stack);

                if let Some(else_) = else_.finalize(values, stack) {
                    if let Some(then) = then {
                        Some(Consumable {
                            leftrec: then.leftrec || else_.leftrec,
                            nullable: then.nullable || else_.nullable,
                        })
                    } else {
                        Some(else_)
                    }
                } else {
                    then
                }
            }
            ImlOp::Loop {
                init,
                condition,
                body,
                consuming,
            } => {
                let mut ret: Option<Consumable> = None;

                for part in [
                    init.finalize(values, stack),
                    condition.finalize(values, stack),
                    body.finalize(values, stack),
                ] {
                    if let Some(part) = part {
                        ret = if let Some(ret) = ret {
                            Some(Consumable {
                                leftrec: ret.leftrec || part.leftrec,
                                nullable: ret.nullable || part.nullable,
                            })
                        } else {
                            Some(part)
                        }
                    }
                }

                *consuming = ret.clone();

                ret
            }

            _ => None,
        }
    }
}

impl std::fmt::Display for ImlOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImlOp::Compileable(p) => write!(f, "{}", p),
            op => write!(f, "Op {:?}", op),
        }
    }
}

impl From<Op> for ImlOp {
    fn from(op: Op) -> Self {
        ImlOp::Op(op)
    }
}
