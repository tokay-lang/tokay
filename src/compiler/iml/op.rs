/*! Intermediate code representation. */

use super::*;
use crate::{Object, Program, RefValue, Value};
use indexmap::{IndexMap, IndexSet};
use num::ToPrimitive;
use std::cell::RefCell;
use std::rc::Rc;

/*
    Todo / Ideas for this module

    - [x] Usage is integrated into ImlOp, eventually by using Rc<RefCell>?
    - [x] Compilable is integrated into ImlOp as full variation
      - [x] Alternation, Sequence, If, Loop
      - [ ] Replace expect, not, peek, repeat by their generic counterparts
        - [ ] Thinking about inline-parselets, whose VM code will be inserted right in place (or already on ImlOp level)
    - [ ] Integrate ImlResult into ImlOp
    - [ ] Finalization must be re-defined, as this is only possible on consumable constructs
      - find left-recursions
      - find nullables
*/

pub type SharedImlOp = Rc<RefCell<ImlOp>>;

/// This struct is required to avoid endless recursion im ImlValue in case of a recursive ImlParselet.
pub struct ImlOpValue(pub ImlValue);

impl std::ops::Deref for ImlOpValue {
    type Target = ImlValue;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for ImlOpValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            ImlValue::Parselet(p) => write!(
                f,
                "Parselet({})",
                p.borrow().name.as_deref().unwrap_or("<unnamed>")
            ),
            _ => self.0.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum ImlOp {
    Nop,                           // Empty operation
    Op(Op),                        // VM Operation
    Shared(SharedImlOp), // Shared ImlOp tree can be shared from various locations during compilation
    Usage(Usage),        // (yet) unresolved usage
    Load(ImlOpValue),    // Qualified value load
    Call(ImlOpValue, usize, bool), // Qualified value call

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

    // v--- below variants are being replaced by Tokay generics as soon as they are implemented ---v //

    // Expect (deprecated!)
    Expect {
        body: Box<ImlOp>,
        msg: Option<String>,
    },

    // Not (deprecated!)
    Not {
        body: Box<ImlOp>,
    },

    // Peek (deprecated!)
    Peek {
        body: Box<ImlOp>,
    },

    // Repeat (deprecated!)
    Repeat {
        body: Box<ImlOp>,
        min: usize,
        max: usize,
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
        Self::Repeat {
            body: Box::new(self),
            min: 0,
            max: 0,
        }
    }

    pub fn into_positive(self) -> Self {
        Self::Repeat {
            body: Box::new(self),
            min: 1,
            max: 0,
        }
    }

    pub fn into_optional(self) -> Self {
        Self::Repeat {
            body: Box::new(self),
            min: 0,
            max: 1,
        }
    }

    pub fn into_peek(self) -> Self {
        Self::Peek {
            body: Box::new(self),
        }
    }

    pub fn into_not(self) -> Self {
        Self::Not {
            body: Box::new(self),
        }
    }

    pub fn into_expect(self, msg: Option<String>) -> Self {
        Self::Expect {
            body: Box::new(self),
            msg,
        }
    }

    pub(super) fn compile(&self, linker: &mut Linker) -> Vec<Op>
/* todo: extend a provided ops rather than returning a Vec */ {
        match self {
            ImlOp::Nop => Vec::new(),
            ImlOp::Op(op) => vec![op.clone()],
            ImlOp::Shared(op) => op.borrow().compile(linker),
            ImlOp::Usage(_) => panic!("Cannot compile ImlOp::Usage"),
            ImlOp::Call(value, args, nargs) => {
                let idx = linker.register_static(value);

                vec![if *args == 0 && !*nargs {
                    Op::CallStatic(idx)
                } else if *args > 0 && !*nargs {
                    Op::CallStaticArg(Box::new((idx, *args)))
                } else {
                    Op::CallStaticArgNamed(Box::new((idx, *args)))
                }]
            }
            ImlOp::Load(value) => {
                vec![linker.push_static(value)]
            }
            ImlOp::Alt { alts } => {
                let mut ret = Vec::new();
                let mut iter = alts.iter();
                let mut jumps = Vec::new();

                while let Some(item) = iter.next() {
                    let alt = item.compile(linker);

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
                    let mut ops = item.compile(linker);

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
                let then = then.compile(linker);

                let backpatch = ret.len();
                ret.push(Op::Nop); // Backpatch operation placeholder

                if *peek {
                    ret.push(Op::Drop)
                }

                let mut jump = then.len() + 1;
                ret.extend(then);

                if !*peek {
                    // Else-part
                    let else_ = else_.compile(linker);

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

                ret.extend(init.compile(linker));

                let mut repeat = condition.compile(linker);
                if !repeat.is_empty() {
                    repeat.push(Op::ForwardIfTrue(2));
                    repeat.push(Op::Break);
                }

                repeat.extend(body.compile(linker));

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
            // DEPRECATED BELOW!!!
            ImlOp::Expect { body, msg } => {
                let code = body.compile(linker);

                let mut ret = vec![Op::Frame(code.len() + 2)];

                ret.extend(code);

                ret.extend(vec![
                    Op::Forward(2),
                    Op::Error(Some(if let Some(msg) = msg {
                        msg.clone()
                    } else {
                        format!("Expecting {:?}", body)
                    })),
                    Op::Close,
                ]);

                ret
            }
            ImlOp::Not { body } => {
                let mut ret = Vec::new();

                let body = body.compile(linker);

                ret.push(Op::Frame(body.len() + 3));
                ret.extend(body);
                ret.push(Op::Close);
                ret.push(Op::Next);

                ret
            }
            ImlOp::Peek { body } => {
                let mut ret = Vec::new();

                ret.push(Op::Frame(0));
                ret.extend(body.compile(linker));
                ret.push(Op::Reset);
                ret.push(Op::Close);

                ret
            }
            ImlOp::Repeat { body, min, max } => {
                let body = body.compile(linker);
                let body_len = body.len();

                let mut ret = Vec::new();

                match (min, max) {
                    (0, 0) => {
                        // Kleene
                        ret.extend(vec![
                            Op::Frame(0),            // The overall capture
                            Op::Frame(body_len + 5), // The fused capture for repetition
                        ]);
                        ret.extend(body); // here comes the body
                        ret.extend(vec![
                            Op::ForwardIfConsumed(2), // When consumed we can commit and jump backward
                            Op::Forward(3),           // otherwise leave the loop
                            Op::Commit,
                            Op::Backward(body_len + 3), // repeat the body
                            Op::Close,
                            Op::Collect(1, 5), // collect only values with severity > 0
                            Op::Close,
                        ]);
                    }
                    (1, 0) => {
                        // Positive
                        ret.push(Op::Frame(0)); // The overall capture
                        ret.extend(body.clone()); // here comes the body for the first time
                        ret.extend(vec![
                            Op::ForwardIfConsumed(2), // If nothing was consumed, then...
                            Op::Next,                 //...reject
                            Op::Frame(body_len + 5),  // The fused capture for repetition
                        ]);
                        ret.extend(body); // here comes the body again inside the repetition
                        ret.extend(vec![
                            Op::ForwardIfConsumed(2), // When consumed we can commit and jump backward
                            Op::Forward(3),           // otherwise leave the loop
                            Op::Commit,
                            Op::Backward(body_len + 3), // repeat the body
                            Op::Close,
                            Op::Collect(1, 5), // collect only values with severity > 0
                            Op::Close,
                        ]);
                    }
                    (0, 1) => {
                        // Optional
                        ret.push(Op::Frame(body_len + 2));
                        ret.extend(body);
                        ret.push(Op::Collect(1, 5)); // collect only values with severity > 0
                        ret.push(Op::Close);
                    }
                    (1, 1) => {}
                    (_, _) => unimplemented!(
                        "ImlOp::Repeat construct with min/max configuration > 1 not implemented yet"
                    ),
                };

                ret
            }
        }
    }

    pub(in crate::compiler) fn finalize(
        &self,
        visited: &mut IndexSet<usize>,
    ) -> Option<Consumable> {
        match self {
            ImlOp::Op(Op::CallStatic(_)) => panic!("May not exists!"),
            ImlOp::Call(ImlOpValue(callee), ..) => {
                match callee {
                    ImlValue::Parselet(callee) => {
                        if let Ok(callee) = callee.try_borrow() {
                            // Close into another parselet?
                            match &callee.consuming {
                                None => None,
                                Some(consuming) => {
                                    if visited.contains(&callee.id()) {
                                        Some(Consumable {
                                            leftrec: false,
                                            nullable: consuming.nullable,
                                        })
                                    } else {
                                        visited.insert(callee.id());

                                        //fixme: Finalize on begin and end as well!
                                        let ret = callee.body.finalize(visited);

                                        visited.remove(&callee.id());

                                        ret
                                    }
                                }
                            }
                        } else {
                            // This is the currently finalized parselet
                            Some(Consumable {
                                leftrec: true,
                                nullable: false,
                            })
                        }
                    }
                    ImlValue::Value(callee) => {
                        if callee.is_consuming() {
                            //println!("{:?} called, which is nullable={:?}", callee, callee.is_nullable());
                            Some(Consumable {
                                leftrec: false,
                                nullable: callee.is_nullable(),
                            })
                        } else {
                            None
                        }
                    }
                    _ => unreachable!(),
                }
            }
            ImlOp::Shared(op) => op.borrow().finalize(visited),
            ImlOp::Alt { alts } => {
                let mut leftrec = false;
                let mut nullable = false;
                let mut consumes = false;

                for alt in alts {
                    if let Some(consumable) = alt.finalize(visited) {
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

                for item in seq {
                    if !nullable {
                        break;
                    }

                    if let Some(consumable) = item.finalize(visited) {
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
                let then = then.finalize(visited);

                if let Some(else_) = else_.finalize(visited) {
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

                for part in [init, condition, body] {
                    let part = part.finalize(visited);

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

                //*consuming = ret.clone();

                ret
            }

            // DEPRECATED BELOW!!!
            ImlOp::Expect { body, .. } => body.finalize(visited),
            ImlOp::Not { body } | ImlOp::Peek { body } => body.finalize(visited),
            ImlOp::Repeat { body, min, max } => {
                if let Some(consumable) = body.finalize(visited) {
                    if *min == 0 {
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

            // default case
            _ => None,
        }
    }

    /** Returns a value to operate with or evaluate during compile-time.

    The function will only return Ok(Value) when the static_expression_evaluation-feature
    is enabled, it is ImlOp::Load and the value is NOT a callable! */
    pub fn get_evaluable_value(&self) -> Result<RefValue, ()> {
        if cfg!(feature = "static_expression_evaluation") {
            if let Self::Load(ImlOpValue(ImlValue::Value(value))) = self {
                if !value.is_callable(true) {
                    return Ok(value.clone().into());
                }
            }
        }

        Err(())
    }
}

/*
impl std::fmt::Debug for ImlOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nop => write!(f, "Nop"),
            Self::Op(op) => write!(f, "{:?}", op),
            Self::Nop => write!(f, "Nop"),
            Self::Nop => write!(f, "Nop"),
    }
}
*/

impl From<Op> for ImlOp {
    fn from(op: Op) -> Self {
        ImlOp::Op(op)
    }
}
