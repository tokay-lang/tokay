/*! Intermediate code representation. */

use super::*;
use crate::reader::Offset;
use crate::utils;
use crate::Compiler;
use crate::{Object, RefValue};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
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

/// Target of a call or load
#[derive(Clone)]
pub enum ImlTarget {
    Identifier(String), // Compile-time identifier (unresolved!)
    Static(ImlValue),   // Compile-time static value
    Local(usize),       // Runtime local value
    Global(usize),      // Runtime global value
}

impl ImlTarget {
    pub fn is_consuming(&self) -> bool {
        match self {
            Self::Identifier(name) => crate::utils::identifier_is_consumable(name),
            Self::Static(value) => value.is_consuming(),
            _ => false, // cannot determine!
        }
    }
}

impl std::fmt::Debug for ImlTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identifier(name) => write!(f, "\"{}\"", name),
            Self::Static(value) => match value {
                ImlValue::Parselet(p) => write!(
                    f,
                    "Parselet({})",
                    p.borrow().name.as_deref().unwrap_or("<unnamed>")
                ),
                _ => value.fmt(f),
            },
            Self::Local(addr) => write!(f, "local@{}", addr),
            Self::Global(addr) => write!(f, "global@{}", addr),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ImlOp {
    Nop,                 // Empty operation
    Op(Op),              // VM Operation
    Shared(SharedImlOp), // Shared ImlOp tree can be shared from various locations during compilation
    Load {
        offset: Option<Offset>,
        target: ImlTarget,
        //copy: bool,  //enforce copy (Op::Sep)
    },
    Call {
        offset: Option<Offset>,
        target: ImlTarget,
        args: Option<(usize, bool)>,
    },

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
    /// Creates a sequence from items, and optimizes stacked, unframed sequences
    pub fn seq(items: Vec<ImlOp>, framed: bool) -> ImlOp {
        let mut seq = Vec::new();

        for item in items {
            if let ImlOp::Seq {
                framed: false,
                seq: items,
            } = item
            {
                seq.extend(items);
            } else {
                seq.push(item);
            }
        }

        match seq.len() {
            0 => ImlOp::Nop,
            1 if !framed => seq.pop().unwrap(),
            _ => ImlOp::Seq { seq, framed },
        }
    }

    /// Load known value
    pub fn load(offset: Option<Offset>, value: ImlValue) -> ImlOp {
        ImlOp::Load {
            offset,
            target: ImlTarget::Static(value),
        }
    }

    /// Load unknown value by name
    pub fn load_by_name(compiler: &mut Compiler, offset: Option<Offset>, name: String) -> ImlOp {
        ImlOp::Load {
            offset,
            target: ImlTarget::Identifier(name),
        }
        .try_resolve(compiler)
    }

    /// Call known value
    pub fn call(offset: Option<Offset>, value: ImlValue, args: Option<(usize, bool)>) -> ImlOp {
        // When args is unset, and the value is not callable without arguments,
        // consider this call as a load.
        if args.is_none() && !value.is_callable(true) {
            // Currently not planned as final
            return Self::load(offset, value);
        }

        // Early recognize call to value which is generally not call-able
        if !value.is_callable(true) && !value.is_callable(false) {
            // Currently not planned as final
            todo!("The value {:?} is generally not callable!", value);
        }

        ImlOp::Call {
            offset,
            target: ImlTarget::Static(value),
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
            offset,
            target: ImlTarget::Identifier(name),
            args,
        }
        .try_resolve(compiler)
    }

    /// Try to resolve immediatelly, otherwise push shared reference to compiler's unresolved ImlOp.
    fn try_resolve(mut self, compiler: &mut Compiler) -> ImlOp {
        if self.resolve(compiler) {
            return self;
        }

        let shared = ImlOp::Shared(Rc::new(RefCell::new(self)));
        compiler.usages.push(shared.clone());
        shared
    }

    pub(in crate::compiler) fn resolve(&mut self, compiler: &mut Compiler) -> bool {
        match self {
            Self::Shared(op) => return op.borrow_mut().resolve(compiler),
            Self::Load { target, .. } | Self::Call { target, .. } => {
                if let ImlTarget::Identifier(name) = target {
                    if let Some(value) = compiler.get_constant(&name) {
                        // Undetermined usages need to remain untouched.
                        if matches!(value, ImlValue::Undetermined(_)) {
                            return false;
                        }

                        *target = ImlTarget::Static(value);
                        return true;
                    } else if let Some(addr) = compiler.get_local(&name) {
                        *target = ImlTarget::Local(addr);
                        return true;
                    } else if let Some(addr) = compiler.get_global(&name) {
                        *target = ImlTarget::Global(addr);
                        return true;
                    }
                }
            }
            _ => {}
        }

        false
    }

    /// Turns ImlOp construct into a kleene (none-or-many) occurence.
    pub fn into_kleene(self) -> Self {
        Self::Repeat {
            body: Box::new(self),
            min: 0,
            max: 0,
        }
    }

    /// Turns ImlOp construct into a positive (one-or-many) occurence.
    pub fn into_positive(self) -> Self {
        Self::Repeat {
            body: Box::new(self),
            min: 1,
            max: 0,
        }
    }

    /// Turns ImlOp construct into an optional (none-or-one) occurence.
    pub fn into_optional(self) -> Self {
        Self::Repeat {
            body: Box::new(self),
            min: 0,
            max: 1,
        }
    }

    /// Turns ImlOp construct into a peeked parser
    pub fn into_peek(self) -> Self {
        Self::Peek {
            body: Box::new(self),
        }
    }

    /// Turns ImlOp construct into a negated parser
    pub fn into_not(self) -> Self {
        Self::Not {
            body: Box::new(self),
        }
    }

    /// Turns ImlOp construct into an expecting parser
    pub fn into_expect(self, msg: Option<String>) -> Self {
        Self::Expect {
            body: Box::new(self),
            msg,
        }
    }

    /// Compile ImlOp construct into Op instructions of the resulting Tokay VM program
    pub(in crate::compiler) fn compile(&self, ops: &mut Vec<Op>, linker: &mut Linker) -> usize {
        let start = ops.len();

        match self {
            ImlOp::Nop => {}
            ImlOp::Op(op) => ops.push(op.clone()),
            ImlOp::Shared(op) => {
                op.borrow().compile(ops, linker);
            }
            ImlOp::Load { offset, target } => {
                if let Some(offset) = offset {
                    ops.push(Op::Offset(Box::new(*offset)));
                }

                ops.push(match target {
                    ImlTarget::Identifier(name) => panic!("Unresolved load of {}", name),
                    ImlTarget::Static(value) => linker.push(value),
                    ImlTarget::Local(idx) => Op::LoadFast(*idx),
                    ImlTarget::Global(idx) => Op::LoadGlobal(*idx),
                });
            }
            ImlOp::Call {
                offset,
                target,
                args,
            } => {
                if let Some(offset) = offset {
                    ops.push(Op::Offset(Box::new(*offset)));
                }

                match target {
                    ImlTarget::Identifier(name) => panic!("Unresolved call to {}", name),
                    ImlTarget::Static(value) => {
                        let idx = linker.register(value);

                        match args {
                            // Qualified call
                            Some((args, nargs)) => {
                                if *args == 0 && !*nargs {
                                    ops.push(Op::CallStatic(idx));
                                } else if *args > 0 && !*nargs {
                                    ops.push(Op::CallStaticArg(Box::new((idx, *args))));
                                } else {
                                    ops.push(Op::CallStaticArgNamed(Box::new((idx, *args))));
                                }
                            }
                            // Call or load
                            None => {
                                if value.is_callable(true) {
                                    ops.push(Op::CallStatic(idx));
                                } else {
                                    ops.push(Op::LoadStatic(idx));
                                }
                            }
                        }

                        return ops.len() - start;
                    }
                    ImlTarget::Local(idx) => ops.push(Op::LoadFast(*idx)),
                    ImlTarget::Global(idx) => ops.push(Op::LoadGlobal(*idx)),
                }

                match args {
                    // Qualified call
                    Some((args, nargs)) => {
                        if *args == 0 && *nargs == false {
                            ops.push(Op::Call);
                        } else if *args > 0 && *nargs == false {
                            ops.push(Op::CallArg(*args));
                        } else {
                            ops.push(Op::CallArgNamed(*args));
                        }
                    }
                    // Call or load
                    None => ops.push(Op::CallOrCopy),
                }
            }
            ImlOp::Alt { alts } => {
                let mut ret = Vec::new();
                let mut iter = alts.iter();
                let mut jumps = Vec::new();

                while let Some(item) = iter.next() {
                    let mut alt = Vec::new();
                    item.compile(&mut alt, linker);

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

                ops.extend(ret);
            }
            ImlOp::Seq { seq, framed } => {
                for item in seq.iter() {
                    item.compile(ops, linker);

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
                }

                // Create a frame and collect in framed mode and when there's more than one operation inside ret.
                if *framed
                    && ops[start..]
                        .iter()
                        .map(|op| if matches!(op, Op::Offset(_)) { 0 } else { 1 })
                        .sum::<usize>()
                        > 1
                {
                    ops.insert(start, Op::Frame(0));
                    ops.push(Op::Collect(0, 5));
                    ops.push(Op::Close);
                }
            }
            ImlOp::If {
                peek,
                test,
                then: then_part,
                else_: else_part,
            } => {
                // Clone on peek
                if *peek {
                    ops.push(Op::Clone);
                }

                let backpatch = ops.len();
                ops.push(Op::Nop); // Backpatch operation placeholder

                if *peek {
                    ops.push(Op::Drop)
                }

                // Then-part
                let mut jump = then_part.compile(ops, linker) + 1;

                if !*peek {
                    let mut else_ops = Vec::new();

                    // Else-part
                    if else_part.compile(&mut else_ops, linker) > 0 {
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
                consuming, // todo: currently always false, which is wrong!
                init,
                condition,
                body,
            } => {
                init.compile(ops, linker);

                let mut repeat = Vec::new();

                if condition.compile(&mut repeat, linker) > 0 {
                    ops.push(Op::ForwardIfTrue(2));
                    ops.push(Op::Break);
                }

                body.compile(&mut repeat, linker);

                ops.push(Op::Loop(
                    repeat.len() + if consuming.is_some() { 3 } else { 2 },
                ));

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
            ImlOp::Expect { body, msg } => {
                let mut expect = Vec::new();
                body.compile(&mut expect, linker);

                ops.push(Op::Frame(expect.len() + 2));

                ops.extend(expect);
                ops.extend(vec![
                    Op::Forward(2),
                    Op::Error(Some(if let Some(msg) = msg {
                        msg.clone()
                    } else {
                        format!("Expecting {:?}", body)
                    })),
                    Op::Close,
                ]);
            }
            ImlOp::Not { body } => {
                let mut body_ops = Vec::new();
                let body_len = body.compile(&mut body_ops, linker);
                ops.push(Op::Frame(body_len + 3));
                ops.extend(body_ops);
                ops.push(Op::Close);
                ops.push(Op::Next);
            }
            ImlOp::Peek { body } => {
                ops.push(Op::Frame(0));
                body.compile(ops, linker);
                ops.push(Op::Reset);
                ops.push(Op::Close);
            }
            ImlOp::Repeat { body, min, max } => {
                let mut body_ops = Vec::new();
                let body_len = body.compile(&mut body_ops, linker);

                match (min, max) {
                    (0, 0) => {
                        // Kleene
                        ops.extend(vec![
                            Op::Frame(0),            // The overall capture
                            Op::Frame(body_len + 5), // The fused capture for repetition
                        ]);
                        ops.extend(body_ops); // here comes the body
                        ops.extend(vec![
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
                        ops.push(Op::Frame(0)); // The overall capture
                        ops.extend(body_ops.clone()); // here comes the body for the first time
                        ops.extend(vec![
                            Op::ForwardIfConsumed(2), // If nothing was consumed, then...
                            Op::Next,                 //...reject
                            Op::Frame(body_len + 5),  // The fused capture for repetition
                        ]);
                        ops.extend(body_ops); // here comes the body again inside the repetition
                        ops.extend(vec![
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
                        ops.push(Op::Frame(body_len + 2));
                        ops.extend(body_ops);
                        ops.push(Op::Collect(1, 5)); // collect only values with severity > 0
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

    /** Finalize ImlOp construct on a grammar's point of view.

    This function must be run inside of a closure on every parselet until no more changes occur.
    */
    pub(in crate::compiler) fn finalize(
        &self,
        visited: &mut HashSet<usize>,
        configs: &mut HashMap<usize, Consumable>,
    ) -> Option<Consumable> {
        match self {
            ImlOp::Shared(op) => op.borrow().finalize(visited, configs),
            ImlOp::Call {
                target: ImlTarget::Static(callee),
                ..
            } => {
                match callee {
                    ImlValue::Parselet(parselet) => {
                        match parselet.try_borrow() {
                            // In case the parselet cannot be borrowed, it is left-recursive!
                            Err(_) => Some(Consumable {
                                leftrec: true,
                                nullable: false,
                            }),
                            Ok(parselet) => {
                                let id = parselet.id();

                                if visited.contains(&id) {
                                    Some(Consumable {
                                        leftrec: false,
                                        nullable: configs[&id].nullable,
                                    })
                                } else {
                                    visited.insert(id);

                                    if !configs.contains_key(&id) {
                                        configs.insert(
                                            id,
                                            Consumable {
                                                leftrec: false,
                                                nullable: false,
                                            },
                                        );
                                    }

                                    //fixme: Finalize on begin and end as well!
                                    let ret = parselet.body.finalize(visited, configs);

                                    visited.remove(&id);

                                    ret
                                }
                            }
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
            ImlOp::Alt { alts } => {
                let mut leftrec = false;
                let mut nullable = false;
                let mut consumes = false;

                for alt in alts {
                    if let Some(consumable) = alt.finalize(visited, configs) {
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

                    if let Some(consumable) = item.finalize(visited, configs) {
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
                let then = then.finalize(visited, configs);

                if let Some(else_) = else_.finalize(visited, configs) {
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
                ..
            } => {
                let mut ret: Option<Consumable> = None;

                for part in [init, condition, body] {
                    let part = part.finalize(visited, configs);

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

                ret
            }

            // DEPRECATED BELOW!!!
            ImlOp::Expect { body, .. } => body.finalize(visited, configs),
            ImlOp::Not { body } | ImlOp::Peek { body } => body.finalize(visited, configs),
            ImlOp::Repeat { body, min, .. } => {
                if let Some(consumable) = body.finalize(visited, configs) {
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

    /// Generic querying function taking a closure that either walks on the tree or stops.
    pub fn walk(&self, func: &mut dyn FnMut(&Self) -> bool) -> bool {
        // Call closure on current ImlOp, break on false return
        if !func(self) {
            return false;
        }

        // Query along ImlOp structure
        match self {
            ImlOp::Shared(op) => op.borrow().walk(func),
            ImlOp::Alt { alts: items } | ImlOp::Seq { seq: items, .. } => {
                for item in items {
                    if !item.walk(func) {
                        return false;
                    }
                }

                true
            }
            ImlOp::If { then, else_, .. } => {
                for i in [&then, &else_] {
                    if !i.walk(func) {
                        return false;
                    }
                }

                true
            }
            ImlOp::Loop {
                init,
                condition,
                body,
                ..
            } => {
                for i in [&init, &condition, &body] {
                    if !i.walk(func) {
                        return false;
                    }
                }

                true
            }
            // DEPRECATED BELOW!!!
            ImlOp::Expect { body, .. }
            | ImlOp::Not { body }
            | ImlOp::Peek { body }
            | ImlOp::Repeat { body, .. } => body.walk(func),

            _ => true,
        }
    }

    pub fn is_consuming(&self) -> bool {
        let mut consuming = false;

        self.walk(&mut |op| {
            match op {
                ImlOp::Call { target, .. } => {
                    if target.is_consuming() {
                        consuming = true;
                        return false; // stop further examination
                    }
                }
                _ => {}
            }

            true
        });

        consuming
    }

    /** Returns a value to operate with or evaluate during compile-time.

    The function will only return Ok(Value) when the static_expression_evaluation-feature
    is enabled, it is ImlOp::Load and the value is NOT a callable! */
    pub fn get_evaluable_value(&self) -> Result<RefValue, ()> {
        if cfg!(feature = "static_expression_evaluation") {
            if let Self::Load {
                target: ImlTarget::Static(ImlValue::Value(value)),
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

impl From<Vec<ImlOp>> for ImlOp {
    fn from(items: Vec<ImlOp>) -> Self {
        ImlOp::seq(items, false)
    }
}
