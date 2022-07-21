/*! Intermediate code representation. */

use super::*;
use crate::{Object, RefValue, Value};
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

#[derive(Debug)]
pub enum ImlOp {
    Nop,                 // Empty operation
    Op(Op),              // VM Operation
    Shared(SharedImlOp), // Shared ImlOp tree can be shared from various locations during compilation
    Usage(Usage),        // (yet) unresolved usage
    Load(ImlValue),      // Qualified value load
    Call(ImlValue),      // Qualified value call

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

    pub(super) fn compile(&self, compiler: &mut Compiler) -> Vec<Op>
/* todo: extend a provided ops rather than returning a Vec */ {
        // Temporary helper function to get clear with ImlOp::Call and ImlOp::Value
        fn push_value(value: &ImlValue, compiler: &mut Compiler) -> Op {
            // Primary value pushes can directly be made by specific VM commands
            if let ImlValue::Value(value) = &value {
                match &*value.borrow() {
                    Value::Void => return Op::PushVoid,
                    Value::Null => return Op::PushNull,
                    Value::True => return Op::PushTrue,
                    Value::False => return Op::PushFalse,
                    Value::Int(i) => match i.to_i64() {
                        Some(0) => return Op::Push0,
                        Some(1) => return Op::Push1,
                        _ => {}
                    },
                    _ => {}
                }
            }

            Op::LoadStatic(compiler.define_value(value.clone()))
        }

        match self {
            ImlOp::Nop => Vec::new(),
            ImlOp::Op(op) => vec![op.clone()],
            ImlOp::Shared(op) => op.borrow().compile(compiler),
            ImlOp::Usage(_) => panic!("Cannot compile ImlOp::Usage"),
            ImlOp::Call(value) => {
                if value.is_callable(true) {
                    vec![Op::CallStatic(compiler.define_value(value.clone()))]
                } else {
                    vec![push_value(value, compiler)]
                }
            }
            ImlOp::Load(value) => {
                vec![push_value(value, compiler)]
            }
            ImlOp::Alt { alts } => {
                let mut ret = Vec::new();
                let mut iter = alts.iter();
                let mut jumps = Vec::new();

                while let Some(item) = iter.next() {
                    let alt = item.compile(compiler);

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
                    let mut ops = item.compile(compiler);

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
                let then = then.compile(compiler);

                let backpatch = ret.len();
                ret.push(Op::Nop); // Backpatch operation placeholder

                if *peek {
                    ret.push(Op::Drop)
                }

                let mut jump = then.len() + 1;
                ret.extend(then);

                if !*peek {
                    // Else-part
                    let else_ = else_.compile(compiler);

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

                ret.extend(init.compile(compiler));

                let mut repeat = condition.compile(compiler);
                if !repeat.is_empty() {
                    repeat.push(Op::ForwardIfTrue(2));
                    repeat.push(Op::Break);
                }

                repeat.extend(body.compile(compiler));

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
                let code = body.compile(compiler);

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

                let body = body.compile(compiler);

                ret.push(Op::Frame(body.len() + 3));
                ret.extend(body);
                ret.push(Op::Close);
                ret.push(Op::Next);

                ret
            }
            ImlOp::Peek { body } => {
                let mut ret = Vec::new();

                ret.push(Op::Frame(0));
                ret.extend(body.compile(compiler));
                ret.push(Op::Reset);
                ret.push(Op::Close);

                ret
            }
            ImlOp::Repeat { body, min, max } => {
                let body = body.compile(compiler);
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

    pub(super) fn finalize(
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
            ImlOp::Shared(op) => op.borrow_mut().finalize(values, stack),
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

            // DEPRECATED BELOW!!!
            ImlOp::Expect { body, .. } | ImlOp::Repeat { body, .. } => body.finalize(values, stack),
            ImlOp::Not { body } | ImlOp::Peek { body } => body.finalize(values, stack),

            // default case
            _ => None,
        }
    }

    /** Returns a value to operate with or evaluate during compile-time.

    The function will only return Ok(Value) when the static_expression_evaluation-feature
    is enabled, it is ImlOp::Load and the value is NOT a callable! */
    pub fn get_evaluable_value(&self) -> Result<RefValue, ()> {
        if cfg!(feature = "static_expression_evaluation") {
            if let Self::Load(ImlValue::Value(value)) = self {
                if !value.is_callable(true) {
                    return Ok(value.clone().into());
                }
            }
        }

        Err(())
    }
}

impl From<Op> for ImlOp {
    fn from(op: Op) -> Self {
        ImlOp::Op(op)
    }
}
