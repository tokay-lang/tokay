/*! Intermediate code representation. */

use super::*;
use crate::reader::Offset;
use crate::utils;
use crate::Compiler;
use crate::{Object, RefValue};

#[derive(Debug, Clone)]
pub(in crate::compiler) enum ImlOp {
    Nop,    // Empty operation
    Op(Op), // VM Operation
    Load {
        offset: Option<Offset>,
        target: ImlValue,
        //copy: bool,  //enforce copy (Op::Sep)
    },
    Call {
        offset: Option<Offset>,
        target: ImlValue,
        args: Option<(usize, bool)>,
    },

    // Alternation (Block) of sequences or ops
    Alt {
        alts: Vec<ImlOp>,
    },

    // Sequence of ops, optionally a collection
    Seq {
        seq: Vec<ImlOp>,
        collection: bool, /* According to these operation's semantics, or when an entire sequence is completely recognized,
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
        iterator: bool,        // Test condition either for void (=true) or bool (=false)
        initial: Box<ImlOp>,   // Initialization
        condition: Box<ImlOp>, // Abort condition
        body: Box<ImlOp>,      // Iterating body
    },

    // v--- below variants are being replaced by Tokay generics as soon as they are implemented ---v //

    // Repeat (deprecated!)
    Repeat {
        body: Box<ImlOp>,
        min: usize,
        max: usize,
    },
}

impl ImlOp {
    /// Creates a sequence from items, and optimizes stacked, unframed sequences
    pub fn seq(items: Vec<ImlOp>, collection: bool) -> ImlOp {
        let mut seq = Vec::new();

        for item in items {
            match item {
                ImlOp::Nop => {}
                ImlOp::Seq {
                    collection: false,
                    seq: items,
                } => seq.extend(items),
                item => seq.push(item),
            }
        }

        match seq.len() {
            0 => ImlOp::Nop,
            1 if !collection => seq.pop().unwrap(),
            _ => ImlOp::Seq { seq, collection },
        }
    }

    /// Load value
    pub fn load(_compiler: &mut Compiler, offset: Option<Offset>, value: ImlValue) -> ImlOp {
        ImlOp::Load {
            offset,
            target: value,
        }
    }

    /// Load unknown value by name
    pub fn load_by_name(compiler: &mut Compiler, offset: Option<Offset>, name: String) -> ImlOp {
        let value = ImlValue::Name { offset, name }.try_resolve(compiler);

        Self::load(compiler, offset.clone(), value)
    }

    /// Call known value
    pub fn call(
        compiler: &mut Compiler,
        offset: Option<Offset>,
        value: ImlValue,
        args: Option<(usize, bool)>,
    ) -> ImlOp {
        // When args is unset, and the value is not callable without arguments,
        // consider this call is a load.
        if args.is_none() && !value.is_callable(true) {
            // Currently not planned as final
            return Self::load(compiler, offset, value);
        }

        if value.is_consuming() {
            compiler.parselet_mark_consuming();
        }

        ImlOp::Call {
            offset,
            target: value,
            args,
        }
    }

    /// Call unknown value by name
    pub fn call_by_name(
        compiler: &mut Compiler,
        offset: Option<Offset>,
        name: String,
        args: Option<(usize, bool)>,
    ) -> ImlOp {
        // Perform early consumable detection depending on identifier's name
        if utils::identifier_is_consumable(&name) {
            compiler.parselet_mark_consuming();
        }

        ImlOp::Call {
            offset: offset.clone(),
            target: ImlValue::Name { offset, name }.try_resolve(compiler),
            args,
        }
    }

    /// Compile ImlOp construct into Op instructions of the resulting Tokay VM program
    pub fn compile_to_vec(
        &self,
        program: &mut ImlProgram,
        current: (&ImlParselet, usize),
    ) -> Vec<Op> {
        let mut ops = Vec::new();
        self.compile(program, current, &mut ops);
        ops
    }

    /// Compile ImlOp construct into Op instructions of the resulting Tokay VM program
    pub fn compile(
        &self,
        program: &mut ImlProgram,
        current: (&ImlParselet, usize),
        ops: &mut Vec<Op>,
    ) -> usize {
        let start = ops.len();

        match self {
            ImlOp::Nop => {}
            ImlOp::Op(op) => ops.push(op.clone()),
            ImlOp::Load { offset, target } => {
                target.compile(program, current, &offset, None, ops);
            }
            ImlOp::Call {
                offset,
                target,
                args,
            } => {
                target.compile(program, current, &offset, Some(*args), ops);
            }
            ImlOp::Alt { alts } => {
                let mut ret = Vec::new();
                let mut iter = alts.iter();
                let mut jumps = Vec::new();
                let mut initial_fuse = None;

                while let Some(item) = iter.next() {
                    let alt = item.compile_to_vec(program, current);

                    // When branch has more than one item, Frame it.
                    if iter.len() > 0 {
                        let consuming = item.is_consuming();
                        let fuse = alt.len() + if consuming { 3 } else { 2 };

                        if initial_fuse.is_none() {
                            initial_fuse = Some(fuse) // this is used for the initial frame
                        } else {
                            ret.push(Op::Fuse(fuse)); // this updates the fuse of the frame
                        }

                        ret.extend(alt);

                        if consuming {
                            // Insert Nop as location for later jump backpatch
                            ret.push(Op::Nop);
                            jumps.push(ret.len() - 1);
                            ret.push(Op::Reset);
                        } else {
                            ret.push(Op::ResetCapture);
                        }
                    } else {
                        ret.extend(alt);
                    }
                }

                // Backpatch remembered jumps
                while let Some(addr) = jumps.pop() {
                    ret[addr] = Op::ForwardIfConsumed(ret.len() - addr);
                }

                // Wrap the entire body in its own frame when more than 1 alternative exists
                if let Some(fuse) = initial_fuse {
                    ret.insert(0, Op::Frame(fuse));
                    ret.push(Op::Close);
                }

                ops.extend(ret);
            }
            ImlOp::Seq { seq, collection } => {
                for item in seq.iter() {
                    item.compile(program, current, ops);
                }

                // Check if the sequence exists of more than one operational instruction
                if *collection
                    && ops[start..]
                        .iter()
                        .map(|op| if matches!(op, Op::Offset(_)) { 0 } else { 1 })
                        .sum::<usize>()
                        > 1
                {
                    ops.insert(start, Op::Frame(0));
                    ops.push(Op::Collect);
                    ops.push(Op::Close);
                }
            }
            ImlOp::If {
                peek,
                test,
                then: then_part,
                else_: else_part,
            } => {
                // Copy on peek
                if *peek {
                    ops.push(Op::Copy(1));
                }

                let backpatch = ops.len();
                ops.push(Op::Nop); // Backpatch operation placeholder

                if *peek {
                    ops.push(Op::Drop)
                }

                // Then-part
                let mut jump = then_part.compile(program, current, ops) + 1;

                if !*peek {
                    let mut else_ops = Vec::new();

                    // Else-part
                    if else_part.compile(program, current, &mut else_ops) > 0 {
                        ops.push(Op::Forward(else_ops.len() + 1));
                        jump += 1;
                        ops.extend(else_ops);
                    }
                } else {
                    jump += 1;
                }

                // Insert the final condition and its failure target.
                if *test {
                    ops[backpatch] = Op::ForwardIfFalse(jump);
                } else {
                    ops[backpatch] = Op::ForwardIfTrue(jump);
                }
            }
            ImlOp::Loop {
                iterator,
                initial,
                condition,
                body,
            } => {
                let consuming: Option<bool> = None; // fixme: Currently not sure if this is an issue.
                let mut repeat = Vec::new();

                initial.compile(program, current, ops);

                if condition.compile(program, current, &mut repeat) > 0 {
                    if *iterator {
                        repeat.push(Op::ForwardIfNotVoid(2));
                    } else {
                        repeat.push(Op::ForwardIfTrue(2));
                    }

                    repeat.push(Op::Break);
                }

                body.compile(program, current, &mut repeat);
                let len = repeat.len() + if consuming.is_some() { 3 } else { 2 };

                ops.push(Op::Loop(len));

                // fixme: consuming flag must be handled differently.
                if consuming.is_some() {
                    ops.push(Op::Fuse(ops.len() - start + 2));
                }

                ops.extend(repeat);
                ops.push(Op::Continue);

                if consuming.is_some() {
                    ops.push(Op::Break);
                }
            }
            // DEPRECATED BELOW!!!
            ImlOp::Repeat { body, min, max } => {
                let mut body_ops = Vec::new();
                let body_len = body.compile(program, current, &mut body_ops);

                match (min, max) {
                    (0, 0) => {
                        // Kleene
                        ops.extend(vec![
                            Op::Frame(0),            // The overall capture
                            Op::Frame(body_len + 6), // The fused capture for repetition
                        ]);
                        ops.extend(body_ops); // here comes the body
                        ops.extend(vec![
                            Op::ForwardIfConsumed(2), // When consumed we can commit and jump backward
                            Op::Forward(4),           // otherwise leave the loop
                            Op::Capture,
                            Op::Extend,
                            Op::Backward(body_len + 4), // repeat the body
                            Op::Close,
                            Op::InCollect,
                            Op::Close,
                        ]);
                    }
                    (1, 0) => {
                        // Positive
                        ops.push(Op::Frame(0)); // The overall capture
                        ops.extend(body_ops.clone()); // here comes the body for the first time
                        ops.extend(vec![
                            Op::ForwardIfConsumed(2), // If nothing was consumed, then...
                            Op::Next,                 //...reject
                            Op::Frame(body_len + 6),  // The fused capture for repetition
                        ]);
                        ops.extend(body_ops); // here comes the body again inside the repetition
                        ops.extend(vec![
                            Op::ForwardIfConsumed(2), // When consumed we can commit and jump backward
                            Op::Forward(4),           // otherwise leave the loop
                            Op::Capture,
                            Op::Extend,
                            Op::Backward(body_len + 4), // repeat the body
                            Op::Close,
                            Op::InCollect,
                            Op::Close,
                        ]);
                    }
                    (0, 1) => {
                        // Optional
                        ops.push(Op::Frame(body_len + 1)); // on error, jump to the collect
                        ops.extend(body_ops);
                        ops.push(Op::InCollect);
                        ops.push(Op::Close);
                    }
                    (1, 1) => {}
                    (_, _) => unimplemented!(
                        "ImlOp::Repeat construct with min/max configuration > 1 not implemented yet"
                    ),
                };
            }
        }

        ops.len() - start
    }

    // Defines the ImlOp's consuming state from point of view as an ImlOp.
    // The ImlOp deeply can still consume, but this is a semantic issue.
    // During code-generation, this function is useful to determine whether
    // the ImlOp is directly consuming or not.
    pub fn is_consuming(&self) -> bool {
        fn walk(op: &ImlOp) -> Option<bool> {
            // Query along ImlOp structure
            match op {
                ImlOp::Call { target, .. } => {
                    if target.is_consuming() {
                        return Some(true);
                    }

                    None
                }
                ImlOp::Op(Op::Next) => Some(true),
                ImlOp::Loop { .. } | ImlOp::If { peek: false, .. } => Some(false),
                ImlOp::Alt { alts: items } | ImlOp::Seq { seq: items, .. } => {
                    for item in items {
                        if let Some(res) = walk(item) {
                            return Some(res);
                        }
                    }

                    None
                }
                ImlOp::If { then, else_, .. } => {
                    for item in [&then, &else_] {
                        if let Some(res) = walk(item) {
                            return Some(res);
                        }
                    }

                    None
                }
                // DEPRECATED BELOW!!!
                ImlOp::Repeat { body, .. } => walk(body),

                _ => None,
            }
        }

        walk(self).unwrap_or(false)
    }

    /** Returns a value to operate with or evaluate during compile-time.

    The function will only return Ok(Value) when the static_expression_evaluation-feature
    is enabled, it is ImlOp::Load and the value is NOT a callable! */
    pub fn get_evaluable_value(&self) -> Result<RefValue, ()> {
        if cfg!(feature = "static_expression_evaluation") {
            if let Self::Load {
                target: ImlValue::Value(value),
                ..
            } = self
            {
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

impl From<Vec<ImlOp>> for ImlOp {
    fn from(items: Vec<ImlOp>) -> Self {
        ImlOp::seq(items, false)
    }
}
