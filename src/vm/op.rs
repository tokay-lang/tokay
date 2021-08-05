use super::*;
use crate::error::Error;
use crate::reader::Offset;
use crate::value::{Dict, Value};

// --- Op ----------------------------------------------------------------------

/**
Atomic operations.

Specifies atomic level operations like running a parsable structure or running
VM code.
*/
#[derive(Debug, Clone)]
pub enum Op {
    Nop,
    Offset(Box<Offset>), // Source offset position for debugging
    Rust(Rust),          // Native rust callback

    // Call
    CallOrCopy,          // Load and eventually call stack element without parameters
    Call,                // Call stack element without parameters
    CallArg(usize),      // Call stack element with sequential parameters
    CallArgNamed(usize), // Call stack element with sequential and named parameters
    CallStatic(usize),   // Call static element without parameters
    CallStaticArg(Box<(usize, usize)>), // Call static element with sequential parameters
    CallStaticArgNamed(Box<(usize, usize)>), // Call static element with sequential and named parameters

    // Jumps
    Fused(Box<(usize, usize)>), // On operation soft reject within abort range, goto abort, otherwise goto succeed

    // Interrupts
    Skip,       // Err(Reject::Skip)
    Next,       // Err(Reject::Next)
    Push,       // Ok(Accept::Push)
    Continue,   // Ok(Accept::Continue)
    LoadPush,   // Ok(Accept::Push) with value
    Break,      // Ok(Accept::Break)
    LoadBreak,  // Ok(Accept::Break) with value
    Accept,     // Ok(Accept::Return)
    LoadAccept, // Ok(Accept::Return) with value
    Repeat,     // Ok(Accept::Repeat)
    LoadRepeat, // Ok(Accept::Repeat) with value
    Reject,     // Ok(Err::Reject)
    LoadExit,   // Exit with errorcode
    Exit,       // Exit with 0

    // Constants
    LoadStatic(usize), // Load static from statics
    Push0,             // Push Integer(0)
    Push1,             // Push Integer(1)
    PushVoid,          // Push Void
    PushNull,          // Push Null
    PushTrue,          // Push True
    PushFalse,         // Push False

    // Variables & Values
    LoadGlobal(usize),
    LoadFast(usize),
    LoadFastCapture(usize),
    LoadCapture,
    LoadAttr,
    LoadIndex,
    //LoadFastAttr(usize),
    StoreGlobal(usize),
    StoreGlobalHold(usize),
    StoreFast(usize),
    StoreFastHold(usize),
    StoreFastCapture(usize),
    StoreFastCaptureHold(usize),
    StoreCapture,
    StoreCaptureHold,

    MakeAlias,             // Make key-value-Capture from last two stack items
    MakeDict(usize),       // Make a Dict from specified amount of key-value-pairs
    MakeCollection(usize), // Either make a List or Dict from specified amount of Captures

    // Operations
    Drop, // drop TOS
    Dup,  // duplicate TOS
    Rot2, // rotate TOS by 2

    Add, // binary add
    Sub, // binary sub
    Mul, // binary mul
    Div, // binary div

    Not, // unary not (! operator)
    Neg, // unary negation (-x operator)

    InlineAdd, // Inline add (+= operator)
    InlineSub, // Inline sub (-= operator)
    InlineMul, // Inline mul (*= operator)
    InlineDiv, // Inline div (/= operator)

    InlineInc, // Inline increment (++x and x++ operators)
    InlineDec, // Inline decrement (--x and x-- operators)

    Equal,        // Compare for equality (== operator)
    NotEqual,     // Compare for unequality (!= operator)
    LowerEqual,   // Compare for lower-equality (<= operator)
    GreaterEqual, // Compare for greater-equality (>= operator)
    Lower,        // Compare for lowerness (< operator)
    Greater,      // Compare for greaterness (> operator)
}

impl Op {
    // Current recursive run function, that will be removed soon.
    pub fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        match self {
            Op::Nop => Ok(Accept::Next),
            Op::Offset(offset) => {
                context.source_offset = Some(**offset);
                Ok(Accept::Skip)
            }
            Op::Rust(f) => f.0(context),

            // Calls
            Op::CallOrCopy => {
                let value = context.pop();
                let value = value.borrow();

                if value.is_callable(false) {
                    // Call the value without parameters
                    value.call(context, 0, None)
                } else {
                    // Push a copy of the value
                    context.push(value.clone().into_refvalue())
                }
            }

            Op::Call => {
                let target = context.pop();
                let target = target.borrow();
                target.call(context, 0, None)
            }

            Op::CallArg(args) => {
                let target = context.pop();
                let target = target.borrow();
                target.call(context, *args, None)
            }

            Op::CallArgNamed(args) => {
                let target = context.pop();
                let target = target.borrow();

                let nargs = Value::from_ref(context.pop()).unwrap();
                target.call(context, *args, Some(nargs.into_dict()))
                //println!("CallArgNamed returns {:?}", ret);
            }

            Op::CallStatic(addr) => context.runtime.program.statics[*addr]
                .borrow()
                .call(context, 0, None),

            Op::CallStaticArg(addr_args) => {
                context.runtime.program.statics[addr_args.0].borrow().call(
                    context,
                    addr_args.1,
                    None,
                )
                //println!("CallStaticArg returns {:?}", ret);
            }

            Op::CallStaticArgNamed(addr_args) => {
                let nargs = Value::from_ref(context.pop()).unwrap();

                context.runtime.program.statics[addr_args.0].borrow().call(
                    context,
                    addr_args.1,
                    Some(nargs.into_dict()),
                )
                //println!("CallStaticArg returns {:?}",
            }

            // Jumps
            Op::Fused(_) => {
                // See below...
                unimplemented!();
            }

            // Execution
            Op::Skip => Err(Reject::Skip),
            Op::Next => Err(Reject::Next),
            Op::Continue => Ok(Accept::Continue),

            Op::Push => Ok(Accept::Push(Capture::Empty)),
            Op::LoadPush => {
                let value = context.pop();
                Ok(Accept::Push(Capture::Value(value, None, 15))) // high severity for override required here
            }

            Op::Break => Ok(Accept::Break(None)),
            Op::LoadBreak => {
                let value = context.pop();
                Ok(Accept::Break(Some(value)))
            }

            Op::Accept => Ok(Accept::Return(None)),
            Op::LoadAccept => {
                let value = context.pop();
                Ok(Accept::Return(Some(value)))
            }
            Op::Repeat => Ok(Accept::Repeat(None)),
            Op::LoadRepeat => {
                let value = context.pop();
                Ok(Accept::Repeat(Some(value)))
            }
            Op::Reject => Err(Reject::Return),
            Op::LoadExit => {
                std::process::exit(context.pop().borrow().to_integer() as i32);
            }
            Op::Exit => std::process::exit(0),

            // Values
            Op::LoadStatic(addr) => {
                let value = &context.runtime.program.statics[*addr];
                context.push(value.borrow().clone().into_refvalue())
            }
            Op::Push0 => context.push(Value::Integer(0).into_refvalue()),
            Op::Push1 => context.push(Value::Integer(1).into_refvalue()),
            Op::PushVoid => context.push(Value::Void.into_refvalue()),
            Op::PushNull => context.push(Value::Null.into_refvalue()),
            Op::PushTrue => context.push(Value::True.into_refvalue()),
            Op::PushFalse => context.push(Value::False.into_refvalue()),

            Op::LoadGlobal(addr) => context.load(*addr),
            Op::LoadFast(addr) => context.load(context.stack_start + *addr),

            Op::LoadFastCapture(index) => {
                let value = context
                    .get_capture(*index)
                    .unwrap_or(Value::Void.into_refvalue());
                context.push(value)
            }

            Op::LoadCapture => {
                let index = context.pop();
                let index = index.borrow();

                match &*index {
                    Value::Addr(_) | Value::Integer(_) | Value::Float(_) => {
                        let value = context
                            .get_capture(index.to_addr())
                            .unwrap_or(Value::Void.into_refvalue());
                        context.push(value)
                    }

                    Value::String(alias) => {
                        let value = context
                            .get_capture_by_name(alias)
                            .unwrap_or(Value::Void.into_refvalue());
                        context.push(value)
                    }

                    _ => Ok(Accept::Next),
                }
            }

            Op::LoadAttr => {
                let attr = context.pop();
                let attr = attr.borrow();
                let value = context.pop();

                match Value::get_attr(value, &attr) {
                    Ok(value) => context.push(value),
                    Err(msg) => Error::new(None, msg).into_reject(),
                }
            }

            Op::LoadIndex => {
                let index = context.pop();
                let index = index.borrow();
                let value = context.pop();
                let value = value.borrow();

                match value.get_index(&index) {
                    Ok(value) => context.push(value),
                    Err(msg) => Error::new(None, msg).into_reject(),
                }
            }

            Op::StoreGlobal(addr) => {
                // todo: bounds checking?
                let value = context.pop();
                context.runtime.stack[*addr] = Capture::Value(value, None, 0);
                Ok(Accept::Next)
            }

            Op::StoreGlobalHold(addr) => {
                // todo: bounds checking?
                let value = context.peek();
                context.runtime.stack[*addr] =
                    Capture::Value(value.borrow().clone().into_refvalue(), None, 0);
                Ok(Accept::Skip)
            }

            Op::StoreFast(addr) => {
                // todo: bounds checking?
                let value = context.pop();
                context.runtime.stack[context.stack_start + *addr] =
                    Capture::Value(value, None, 0);
                Ok(Accept::Next)
            }

            Op::StoreFastHold(addr) => {
                // todo: bounds checking?
                let value = context.peek();
                context.runtime.stack[context.stack_start + *addr] =
                    Capture::Value(value.borrow().clone().into_refvalue(), None, 0);
                Ok(Accept::Skip)
            }

            Op::StoreFastCapture(index) => {
                let value = context.pop();

                context.set_capture(*index, value);
                Ok(Accept::Next)
            }

            Op::StoreFastCaptureHold(index) => {
                let value = context.peek();

                context.set_capture(*index, value.borrow().clone().into_refvalue());
                Ok(Accept::Skip)
            }

            Op::StoreCapture | Op::StoreCaptureHold => {
                let index = context.pop();
                let index = index.borrow();

                match &*index {
                    Value::Addr(_) | Value::Integer(_) | Value::Float(_) => {
                        if matches!(self, Op::StoreCapture) {
                            let value = context.pop();
                            context.set_capture(index.to_addr(), value);
                            Ok(Accept::Next)
                        } else {
                            let value = context.peek();
                            context.set_capture(
                                index.to_addr(),
                                value.borrow().clone().into_refvalue(),
                            );
                            Ok(Accept::Skip)
                        }
                    }

                    Value::String(alias) => {
                        if matches!(self, Op::StoreCapture) {
                            let value = context.pop();
                            context.set_capture_by_name(alias, value);
                            Ok(Accept::Next)
                        } else {
                            let value = context.peek();
                            context.set_capture_by_name(
                                alias,
                                value.borrow().clone().into_refvalue(),
                            );
                            Ok(Accept::Skip)
                        }
                    }

                    _ => Ok(Accept::Next),
                }
            }

            Op::MakeAlias => {
                let name = context.pop();
                let name = name.borrow();

                match context.runtime.stack.last_mut().unwrap() {
                    Capture::Range(_, alias, ..) | Capture::Value(_, alias, ..) => {
                        *alias = Some(name.to_string());
                    }

                    empty => {
                        *empty = Capture::Value(
                            Value::Void.into_refvalue(),
                            Some(name.to_string()),
                            0,
                        );
                    }
                }

                Ok(Accept::Skip)
            }

            Op::MakeDict(count) => {
                let mut dict = Dict::new();

                for _ in 0..*count {
                    let key = context.pop();
                    let key = key.borrow();

                    let value = context.pop();
                    dict.insert(key.to_string(), value);
                }

                context.push(Value::Dict(Box::new(dict)).into_refvalue())
            }

            Op::MakeCollection(count) => {
                if let Ok(Some(value)) =
                    context.collect(context.runtime.stack.len() - count, false, false, false, 0)
                {
                    context.push(value)
                } else {
                    Ok(Accept::Next)
                }
            }

            Op::Drop => {
                context.pop();
                Ok(Accept::Skip)
            }

            Op::Dup => {
                let value = context.peek();
                let value = value.borrow();
                context.push(value.clone().into_refvalue())
            }

            Op::Rot2 => {
                let a = context.runtime.stack.pop().unwrap();
                let b = context.runtime.stack.pop().unwrap();

                context.runtime.stack.push(a);
                context.runtime.stack.push(b);

                Ok(Accept::Skip)
            }

            // Operations
            Op::Add | Op::Sub | Op::Mul | Op::Div => {
                let b = context.pop();
                let a = context.pop();

                /*
                println!("{:?}", self);
                println!("a = {:?}", a);
                println!("b = {:?}", b);
                */

                let c = match self {
                    Op::Add => a.borrow().add(&*b.borrow())?.into_refvalue(),
                    Op::Sub => a.borrow().sub(&*b.borrow())?.into_refvalue(),
                    Op::Mul => a.borrow().mul(&*b.borrow())?.into_refvalue(),
                    Op::Div => a.borrow().div(&*b.borrow())?.into_refvalue(),
                    _ => unimplemented!("Unimplemented operator"),
                };

                context.push(c)
            }

            Op::Equal
            | Op::NotEqual
            | Op::LowerEqual
            | Op::GreaterEqual
            | Op::Lower
            | Op::Greater => {
                let b = context.pop();
                let a = context.pop();

                //println!("{:?}", self);
                //println!("a = {:?}", a);
                //println!("b = {:?}", b);

                let c = match self {
                    Op::Equal => &*a.borrow() == &*b.borrow(),
                    Op::NotEqual => &*a.borrow() != &*b.borrow(),
                    Op::LowerEqual => &*a.borrow() <= &*b.borrow(),
                    Op::GreaterEqual => &*a.borrow() >= &*b.borrow(),
                    Op::Lower => &*a.borrow() < &*b.borrow(),
                    Op::Greater => &*a.borrow() > &*b.borrow(),

                    _ => unimplemented!("Unimplemented operator"),
                };

                //println!("c = {:?}", c);

                context.push((if c { Value::True } else { Value::False }).into_refvalue())
            }

            Op::Not => {
                let value = context.pop().borrow().not()?.into_refvalue();
                context.push(value)
            }
            Op::Neg => {
                let value = context.pop().borrow().neg()?.into_refvalue();
                context.push(value)
            }
            Op::InlineAdd | Op::InlineSub | Op::InlineMul | Op::InlineDiv => {
                let b = context.pop();
                let value = context.pop();
                let mut value = value.borrow_mut();

                /*
                println!("{:?}", self);
                println!("a = {:?}", a);
                println!("b = {:?}", b);
                */

                *value = match self {
                    Op::InlineAdd => value.add(&*b.borrow())?,
                    Op::InlineSub => value.sub(&*b.borrow())?,
                    Op::InlineMul => value.mul(&*b.borrow())?,
                    Op::InlineDiv => value.div(&*b.borrow())?,
                    _ => unimplemented!("Unimplemented operator"),
                };

                context.push(value.clone().into_refvalue())
            }

            Op::InlineInc => {
                let value = context.pop();
                let mut value = value.borrow_mut();

                *value = value.add(&Value::Integer(1))?; // todo: perform inc by bit-shift
                context.push(value.clone().into_refvalue())
            }

            Op::InlineDec => {
                let value = context.pop();
                let mut value = value.borrow_mut();

                *value = value.sub(&Value::Integer(1))?; // todo: perform dec by bit-shift
                context.push(value.clone().into_refvalue())
            }
        }
    }

    pub fn execute(ops: &[Op], context: &mut Context) -> Result<Accept, Reject> {
        #[derive(Debug)]
        struct Fuse {
            capture_start: usize,
            reader_start: Offset,
            until: usize,
            succeed: usize
        }

        impl Fuse {
            fn new(context: &Context, until: usize, succeed: usize) -> Fuse {
                assert!(until < succeed);
                Fuse {
                    capture_start: context.runtime.stack.len(),
                    reader_start: context.runtime.reader.tell(),
                    until,
                    succeed
                }
            }
        }

        if ops.len() == 0 {
            return Ok(Accept::Skip);
        }

        // Fuses
        let mut fuses: Vec<Fuse> = vec![
            Fuse::new(context, ops.len() - 1, ops.len())
        ];

        // Current fuse range for fast access;
        // Must be updated whenever a fuse is added or removed
        let mut fused = ops.len() - 1;

        let mut ip = 0;  // Instruction pointer

        loop {
            let op = &ops[ip];

            if context.runtime.debug > 2 {
                context.debug(&format!(
                    "{:03}:{} @ {:04} (fused {})",
                    ip, op,
                    context.runtime.reader.tell().offset,
                    fused
                ));

                if context.runtime.debug > 6 {
                    for i in 0..context.runtime.stack.len() {
                        context.debug(&format!(" {}: {:?}", i, context.runtime.stack[i]));
                    }
                }
            }

            let state = match op {
                Op::Fused(until_succeed) => {
                    fused = ip + until_succeed.0;
                    fuses.push(Fuse::new(context, fused, ip + until_succeed.1));
                    Ok(Accept::Skip)
                }

                op => op.run(context)
            };

            match state {
                Ok(accept) => {
                    match accept {
                        Accept::Next => context.runtime.stack.push(Capture::Empty),
                        Accept::Skip => {},
                        Accept::Push(capture) => context.runtime.stack.push(capture),
                        other => break Ok(other)
                    }

                    // When fused block is left, restore previous fuse.
                    if ip == fused {
                        let fuse = fuses.pop().unwrap();

                        while let Some(fuse) = fuses.last() {
                            fused = fuse.until;
                            if ip < fused {
                                break
                            }
                        }

                        if fuses.len() == 0 {
                            break match context.collect(fuse.capture_start, false, true, true, 0) {
                                Err(capture) => Ok(Accept::Push(capture)),
                                Ok(Some(value)) => Ok(Accept::Push(Capture::Value(value, None, 5))),
                                Ok(None) => Ok(Accept::Next),
                            }
                        }

                        // If input was consumed, succeed
                        if fuse.reader_start == context.runtime.reader.tell() {
                            println!("ip at end of fused area, but no input consumed");
                            context.runtime.stack.truncate(fuse.capture_start);
                            context.runtime.reader.reset(fuse.reader_start);
                        }
                        else {
                            println!("ip at end of fused area, going to {}, new fused {}", fuse.succeed, fused);
                            ip = fuse.succeed;
                        }
                    }
                }

                Err(Reject::Next) => {
                    // When fused block soft-rejects, restore previous context
                    let fuse = fuses.pop().unwrap();

                    println!("fuse until {}", fuse.until);

                    if let Some(fuse) = fuses.last() {
                        fused = fuse.until;
                    }
                    else {
                        break state
                    }

                    context.runtime.stack.truncate(fuse.capture_start);
                    context.runtime.reader.reset(fuse.reader_start);
                    ip = fuse.until + 1;

                    assert!(fuses.len() > 0);
                    continue
                }

                other => break other
            }

            ip += 1;
        }
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Rust(_) => write!(f, "{{rust-function}}"),
            op => write!(f, "{:?}", op),
        }
    }
}

#[derive(Clone)]
pub struct Rust(pub fn(&mut Context) -> Result<Accept, Reject>);

impl std::fmt::Debug for Rust {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{rust-function}}")
    }
}
