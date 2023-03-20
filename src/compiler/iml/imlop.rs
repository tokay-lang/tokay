/*! Intermediate code representation. */

use super::*;
use crate::reader::Offset;
use crate::utils;
use crate::Compiler;
use crate::Error;
use crate::{Object, RefValue};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub(in crate::compiler) type SharedImlOp = Rc<RefCell<ImlOp>>;

#[derive(Debug, Clone)]
pub(in crate::compiler) enum ImlOp {
    Nop,                 // Empty operation
    Op(Op),              // VM Operation
    Shared(SharedImlOp), // Shared ImlOp tree can be shared from various locations during compilation
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

    /// Load known value
    pub fn load(offset: Option<Offset>, value: ImlValue) -> ImlOp {
        ImlOp::Load {
            offset,
            target: value,
        }
    }

    /// Load unknown value by name
    pub fn load_by_name(compiler: &mut Compiler, offset: Option<Offset>, name: String) -> ImlOp {
        ImlOp::Load {
            offset,
            target: ImlValue::Unknown(name),
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
            offset,
            target: ImlValue::Unknown(name),
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
                if let ImlValue::Unknown(name) = target {
                    if let Some(value) = compiler.get_constant(&name) {
                        // In case this is a generic, the value is resolved to a generic for later dispose
                        if matches!(value, ImlValue::Undefined(_)) {
                            *target = ImlValue::Undefined(name.clone());
                        } else {
                            *target = value;
                        }

                        return true;
                    } else if let Some(addr) = compiler.get_local(&name) {
                        *target = ImlValue::Local(addr);
                        return true;
                    } else if let Some(addr) = compiler.get_global(&name) {
                        *target = ImlValue::Global(addr);
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
    pub fn into_expect(self, mut msg: Option<String>) -> Self {
        // When no msg is provided, generate a message from the next consumables in range!
        // This got a bit out of hand, and should be done later via something like a FIRST() attribute on parselet.
        // Generally, this code becomes unnecessary when the Expect<P> generic is made available (see #10 for details)
        if msg.is_none() {
            fn get_expect(op: &ImlOp) -> Option<String> {
                match op {
                    ImlOp::Call { target, .. } | ImlOp::Load { target, .. }
                        if target.is_consuming() =>
                    {
                        Some(format!("{:?}", target).to_string())
                    }
                    ImlOp::Seq { seq, .. } => {
                        let mut txt = None;

                        for item in seq {
                            item.walk(&mut |op| {
                                txt = get_expect(op);
                                !txt.is_some()
                            });

                            if txt.is_some() {
                                break;
                            }
                        }

                        txt
                    }
                    ImlOp::Alt { alts, .. } => {
                        let mut all_txt = Vec::new();

                        for item in alts {
                            let mut txt = None;

                            item.walk(&mut |op| {
                                txt = get_expect(op);
                                !txt.is_some()
                            });

                            if let Some(txt) = txt {
                                all_txt.push(txt);
                            }
                        }

                        if all_txt.is_empty() {
                            None
                        } else {
                            Some(all_txt.join(" or "))
                        }
                    }
                    _ => None,
                }
            }

            self.walk(&mut |op| {
                msg = get_expect(op);
                !msg.is_some()
            });

            if let Some(txt) = msg {
                msg = Some(format!("Expecting {}", txt).to_string())
            }
        }

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
                    ImlValue::Unknown(name) => {
                        linker.errors.push(Error::new(
                            *offset,
                            format!("Use of unresolved symbol '{}'", name),
                        ));

                        Op::Nop
                    }
                    ImlValue::Undefined(name) => {
                        unreachable!("Use of undefined symbol '{}'", name)
                    }
                    ImlValue::Local(idx) => Op::LoadFast(*idx),
                    ImlValue::Global(idx) => Op::LoadGlobal(*idx),
                    value => linker.push(value),
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
                    ImlValue::Unknown(name) => {
                        linker.errors.push(Error::new(
                            *offset,
                            format!("Call to unresolved symbol '{}'", name),
                        ));
                    }
                    ImlValue::Undefined(name) => {
                        unreachable!("Call to undefined symbol '{}' may not occur", name)
                    }
                    ImlValue::Local(idx) => ops.push(Op::LoadFast(*idx)),
                    ImlValue::Global(idx) => ops.push(Op::LoadGlobal(*idx)),
                    value => {
                        // When value is a parselet, check for accepted constant configuration
                        if let ImlValue::Parselet {
                            parselet,
                            constants,
                        } = value
                        {
                            let parselet = parselet.borrow();

                            if !parselet.constants.is_empty() {
                                let mut required = Vec::new();

                                for (name, default) in &parselet.constants {
                                    if matches!(default, ImlValue::Void)
                                        && !constants.contains_key(name)
                                    {
                                        required.push(name.to_string());
                                    }
                                }

                                if !required.is_empty() {
                                    linker.errors.push(Error::new(
                                        offset.clone(),
                                        format!(
                                            "Missing generic configuration on call to '{}<{}>'",
                                            value,
                                            required.join(", ")
                                        ),
                                    ));

                                    return 0;
                                }
                            }
                        }

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
                let mut initial_fuse = None;

                while let Some(item) = iter.next() {
                    let mut alt = Vec::new();
                    item.compile(&mut alt, linker);

                    // When branch has more than one item, Frame it.
                    if iter.len() > 0 {
                        let fuse = alt.len() + if item.is_consuming() { 3 } else { 2 };

                        if initial_fuse.is_none() {
                            initial_fuse = Some(fuse) // this is used for the initial frame
                        } else {
                            ret.push(Op::Fuse(fuse)); // this updates the fuse of the frame
                        }

                        ret.extend(alt);

                        if item.is_consuming() {
                            // Insert Nop as location for later jump backpatch
                            ret.push(Op::Nop);
                            jumps.push(ret.len() - 1);
                        }

                        ret.push(Op::Reset);
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
                    item.compile(ops, linker);
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
                iterator,
                initial,
                condition,
                body,
            } => {
                let consuming: Option<bool> = None; // fixme: Currently not sure if this is an issue.
                let mut repeat = Vec::new();

                initial.compile(ops, linker);

                if condition.compile(&mut repeat, linker) > 0 {
                    if *iterator {
                        repeat.push(Op::ForwardIfNotVoid(2));
                    } else {
                        repeat.push(Op::ForwardIfTrue(2));
                    }

                    repeat.push(Op::Break);
                }

                body.compile(&mut repeat, linker);
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
                ops.push(Op::Close);
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
            ImlOp::Call { target: callee, .. } => {
                match callee {
                    ImlValue::Parselet {
                        parselet,
                        constants,
                    } if constants.is_empty() => {
                        match parselet.try_borrow() {
                            // In case the parselet cannot be borrowed, it is left-recursive!
                            Err(_) => Some(Consumable {
                                leftrec: true,
                                nullable: false,
                            }),
                            // Otherwise dive into this parselet...
                            Ok(parselet) => {
                                // ... only if it's generally flagged to be consuming.
                                if !parselet.consuming {
                                    return None;
                                }

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
                    _ => None,
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
                initial,
                condition,
                body,
                ..
            } => {
                let mut ret: Option<Consumable> = None;

                for part in [initial, condition, body] {
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
                initial,
                condition,
                body,
                ..
            } => {
                for i in [&initial, &condition, &body] {
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
                ImlOp::Op(Op::Next) => {
                    consuming = true;
                    return false; // stop further examination
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
