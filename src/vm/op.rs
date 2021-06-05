use super::*;
use crate::error::Error;
use crate::reader::Offset;
use crate::value::{Dict, RefValue, Value};

// --- Op ----------------------------------------------------------------------

/**
Atomic operations.

Specifies atomic level operations like running a parsable structure or running
VM code.
*/
#[derive(Debug)]
pub enum Op {
    Nop,

    Usage(usize),              // (yet) unresolved usage
    Offset(Box<Offset>),       // Source offset position for debugging
    Runable(Box<dyn Runable>), // Runable item

    // Call
    CallOrCopy,          // Load and eventually call stack element without parameters
    Call,                // Call stack element without parameters
    CallArg(usize),      // Call stack element with sequential parameters
    CallArgNamed(usize), // Call stack element with sequential and named parameters
    CallStatic(usize),   // Call static element without parameters
    CallStaticArg(Box<(usize, usize)>), // Call static element with sequential parameters
    CallStaticArgNamed(Box<(usize, usize)>), // Call static element with sequential and named parameters

    // Interrupts
    Skip,
    Accept,
    LoadAccept,
    Repeat,
    LoadRepeat,
    Reject,

    // Constants
    LoadStatic(usize), // Load static from statics
    Push0,             // Push Integer(0)
    Push1,             // Push Integer(01
    PushVoid,          // Push Void
    PushNull,          // Push Null
    PushTrue,          // Push True
    PushFalse,         // Push False

    // Variables & Values
    LoadGlobal(usize),
    LoadFast(usize),
    LoadFastCapture(usize),
    LoadCapture,
    //LoadAttr,
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

    IAdd, // Inline add (+= operator)
    ISub, // Inline sub (-= operator)
    IMul, // Inline mul (*= operator)
    IDiv, // Inline div (/= operator)

    IInc, // Inline increment (++x and x++ operators)
    IDec, // Inline decrement (--x and x-- operators)

    Equal,        // Compare for equality (== operator)
    NotEqual,     // Compare for unequality (!= operator)
    LowerEqual,   // Compare for lower-equality (<= operator)
    GreaterEqual, // Compare for greater-equality (>= operator)
    Lower,        // Compare for lowerness (< operator)
    Greater,      // Compare for greaterness (> operator)

    LogicalAnd, // Logical and (&& operator)
    LogicalOr,  // Logical or (|| operator)

    // Flow
    If(Box<(Op, Option<Op>)>),
}

impl Op {
    pub fn from_vec(ops: Vec<Op>) -> Self {
        match ops.len() {
            0 => Op::Nop,
            1 => ops.into_iter().next().unwrap(),
            _ => Sequence::new(ops),
        }
    }

    pub fn into_box(self) -> Box<Self> {
        Box::new(self)
    }

    pub fn into_kleene(self) -> Self {
        Repeat::kleene(self)
    }

    pub fn into_positive(self) -> Self {
        Repeat::positive(self)
    }

    pub fn into_optional(self) -> Self {
        Repeat::optional(self)
    }
}

impl Runable for Op {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        /*
        if context.runtime.debug {
            println!("--- {:?} @ {} ---", self, context.runtime.reader.tell().offset);
            for i in 0..context.runtime.stack.len() {
                println!("  {}: {:?}", i, context.runtime.stack[i]);
            }
        }
        */

        match self {
            Op::Nop => Ok(Accept::Next),
            Op::Usage(_) => panic!(
                "{:?} can't be run; Trying to run an unresolved program?",
                self
            ),
            Op::Offset(offset) => {
                context.source_offset = Some(**offset);
                Ok(Accept::Skip)
            }

            Op::Runable(runable) => runable.run(context),

            // Calls
            Op::CallOrCopy => {
                let value = context.pop();
                let value = value.borrow();
                if value.is_callable(false) {
                    value.call(context, 0, None)
                } else {
                    Ok(Accept::Push(Capture::Value(
                        value.clone().into_refvalue(),
                        10,
                    )))
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
                //println!("CallArg returns {:?}", ret);
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

            // Execution
            Op::Skip => Err(Reject::Skip),
            Op::Accept => Ok(Accept::Return(None)),
            Op::LoadAccept => {
                let value = context.pop();
                Ok(Accept::Return(Some(value.clone())))
            }
            Op::Repeat => Ok(Accept::Repeat(None)),
            Op::LoadRepeat => {
                let value = context.pop();
                Ok(Accept::Repeat(Some(value.clone())))
            }
            Op::Reject => Err(Reject::Return),

            // Values
            Op::LoadStatic(addr) => Ok(Accept::Push(Capture::Value(
                context.runtime.program.statics[*addr].clone(),
                10,
            ))),
            Op::Push0 => Ok(Accept::Push(Capture::Value(
                Value::Integer(0).into_refvalue(), // lazy_static?
                10,
            ))),

            Op::Push1 => Ok(Accept::Push(Capture::Value(
                Value::Integer(1).into_refvalue(), // lazy_static?
                10,
            ))),

            Op::PushVoid => Ok(Accept::Push(Capture::Value(
                Value::Void.into_refvalue(),
                10,
            ))),

            Op::PushNull => Ok(Accept::Push(Capture::Value(
                Value::Null.into_refvalue(),
                10,
            ))),

            Op::PushTrue => Ok(Accept::Push(Capture::Value(
                Value::True.into_refvalue(),
                10,
            ))),

            Op::PushFalse => Ok(Accept::Push(Capture::Value(
                Value::False.into_refvalue(),
                10,
            ))),

            Op::LoadGlobal(addr) => Ok(Accept::Push(Capture::Value(
                context.runtime.stack[*addr].as_value(&context.runtime),
                10,
            ))),

            Op::LoadFast(addr) => Ok(Accept::Push(Capture::Value(
                context.runtime.stack[context.stack_start + *addr].as_value(&context.runtime),
                10,
            ))),

            Op::LoadFastCapture(index) => {
                let value = context
                    .get_capture(*index)
                    .unwrap_or(Value::Void.into_refvalue());

                Ok(Accept::Push(Capture::Value(value, 10)))
            }

            Op::LoadCapture => {
                let index = context.pop();
                let index = index.borrow();

                match &*index {
                    Value::Addr(_) | Value::Integer(_) | Value::Float(_) => {
                        let value = context
                            .get_capture(index.to_addr())
                            .unwrap_or(Value::Void.into_refvalue());

                        Ok(Accept::Push(Capture::Value(value, 10)))
                    }

                    Value::String(alias) => {
                        let value = context
                            .get_capture_by_name(alias)
                            .unwrap_or(Value::Void.into_refvalue());

                        Ok(Accept::Push(Capture::Value(value, 10)))
                    }

                    _ => Ok(Accept::Next),
                }
            }

            Op::LoadIndex => {
                let attr = context.pop();
                let attr = attr.borrow();
                let value = context.pop();
                let value = value.borrow();

                match value.get_index(&attr) {
                    Ok(value) => Ok(Accept::Push(Capture::Value(value, 10))),
                    Err(msg) => Error::new(None, msg).into_reject(),
                }
            }

            Op::StoreGlobal(addr) => {
                // todo: bounds checking?
                let value = context.pop();
                context.runtime.stack[*addr] = Capture::Value(value, 10);
                Ok(Accept::Next)
            }

            Op::StoreFast(addr) => {
                // todo: bounds checking?
                let value = context.pop();
                context.runtime.stack[context.stack_start + *addr] = Capture::Value(value, 10);
                Ok(Accept::Next)
            }

            Op::StoreGlobalHold(addr) => {
                // todo: bounds checking?
                let value = context.pop();
                context.runtime.stack[*addr] = Capture::Value(value.clone(), 10);
                Ok(Accept::Push(Capture::Value(value, 10)))
            }

            Op::StoreFastHold(addr) => {
                // todo: bounds checking?
                let value = context.pop();
                context.runtime.stack[context.stack_start + *addr] =
                    Capture::Value(value.clone(), 10);
                Ok(Accept::Push(Capture::Value(value, 10)))
            }

            Op::StoreFastCapture(index) => {
                let value = context.pop();

                context.set_capture(*index, value);
                Ok(Accept::Next)
            }

            Op::StoreFastCaptureHold(index) => {
                let value = context.pop();

                context.set_capture(*index, value.clone());
                Ok(Accept::Push(Capture::Value(value, 10)))
            }

            Op::StoreCapture | Op::StoreCaptureHold => {
                let index = context.pop();
                let index = index.borrow();

                match &*index {
                    Value::Addr(_) | Value::Integer(_) | Value::Float(_) => {
                        if matches!(self, Op::StoreCapture) {
                            Op::StoreFastCapture(index.to_addr()).run(context)
                        } else {
                            Op::StoreFastCaptureHold(index.to_addr()).run(context)
                        }
                    }

                    Value::String(alias) => {
                        let value = context.pop();
                        context.set_capture_by_name(alias, value.clone());

                        if matches!(self, Op::StoreCapture) {
                            Ok(Accept::Next)
                        } else {
                            Ok(Accept::Push(Capture::Value(value, 10)))
                        }
                    }

                    _ => Ok(Accept::Next),
                }
            }

            Op::MakeAlias => {
                let value = context.pop();
                let alias = context.pop();
                let alias = alias.borrow();

                Ok(Accept::Push(Capture::Named(
                    Box::new(Capture::Value(value, 5)),
                    alias.to_string(),
                )))
            }

            Op::MakeDict(count) => {
                let mut dict = Dict::new();

                for _ in 0..*count {
                    let key = context.pop();
                    let key = key.borrow();

                    let value = context.pop();
                    dict.insert(key.to_string(), value);
                }

                Ok(Accept::Push(Capture::from_value(
                    Value::Dict(Box::new(dict)).into_refvalue(),
                )))
            }

            Op::MakeCollection(count) => {
                if let Some(capture) =
                    context.collect(context.runtime.stack.len() - count, false, false, 0, 10)
                {
                    Ok(Accept::Push(capture))
                } else {
                    Ok(Accept::Next)
                }
            }

            Op::Drop => {
                context.pop();
                Ok(Accept::Skip)
            }

            Op::Dup => {
                let value = context
                    .runtime
                    .stack
                    .last()
                    .unwrap()
                    .as_value(&context.runtime);
                let value = value.borrow();
                Ok(Accept::Push(Capture::Value(
                    value.clone().into_refvalue(),
                    10,
                )))
            }

            Op::Rot2 => {
                let a = context.pop();
                let b = context.pop();
                context.push(a);
                context.push(b);
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
                    Op::Add => (&*a.borrow() + &*b.borrow()).into_refvalue(),
                    Op::Sub => (&*a.borrow() - &*b.borrow()).into_refvalue(),
                    Op::Mul => (&*a.borrow() * &*b.borrow()).into_refvalue(),
                    Op::Div => (&*a.borrow() / &*b.borrow()).into_refvalue(),
                    _ => unimplemented!("Unimplemented operator"),
                };

                Ok(Accept::Push(Capture::Value(c, 10)))
            }

            Op::Equal
            | Op::NotEqual
            | Op::LowerEqual
            | Op::GreaterEqual
            | Op::Lower
            | Op::Greater
            | Op::LogicalAnd
            | Op::LogicalOr => {
                let b = context.pop();
                let a = context.pop();

                /*
                println!("{:?}", self);
                println!("a = {:?}", a);
                println!("b = {:?}", b);
                */

                let c = match self {
                    Op::Equal => &*a.borrow() == &*b.borrow(),
                    Op::NotEqual => &*a.borrow() != &*b.borrow(),
                    Op::LowerEqual => &*a.borrow() <= &*b.borrow(),
                    Op::GreaterEqual => &*a.borrow() >= &*b.borrow(),
                    Op::Lower => &*a.borrow() < &*b.borrow(),
                    Op::Greater => &*a.borrow() > &*b.borrow(),
                    Op::LogicalAnd => a.borrow().is_true() && b.borrow().is_true(),
                    Op::LogicalOr => a.borrow().is_true() || b.borrow().is_true(),

                    _ => unimplemented!("Unimplemented operator"),
                };

                Ok(Accept::Push(Capture::Value(
                    (if c { Value::True } else { Value::False }).into_refvalue(),
                    10,
                )))
            }

            Op::Not => Ok(Accept::Push(Capture::Value(
                (!&*context.pop().borrow()).into_refvalue(),
                10,
            ))),

            Op::Neg => Ok(Accept::Push(Capture::Value(
                (-&*context.pop().borrow()).into_refvalue(),
                10,
            ))),

            Op::IAdd | Op::ISub | Op::IMul | Op::IDiv => {
                let b = context.pop();
                let value = context.pop();
                let mut value = value.borrow_mut();

                /*
                println!("{:?}", self);
                println!("a = {:?}", a);
                println!("b = {:?}", b);
                */

                *value = match self {
                    Op::IAdd => (&*value + &*b.borrow()),
                    Op::ISub => (&*value - &*b.borrow()),
                    Op::IMul => (&*value * &*b.borrow()),
                    Op::IDiv => (&*value / &*b.borrow()),
                    _ => unimplemented!("Unimplemented operator"),
                };

                Ok(Accept::Push(Capture::Value(
                    value.clone().into_refvalue(),
                    10,
                )))
            }

            Op::IInc => {
                let value = context.pop();
                let mut value = value.borrow_mut();

                *value = &*value + &Value::Integer(1); // lazy_static?
                context.push(value.clone().into_refvalue());

                Ok(Accept::Skip)
            }

            Op::IDec => {
                let value = context.pop();
                let mut value = value.borrow_mut();

                *value = &*value - &Value::Integer(1); // lazy_static?
                context.push(value.clone().into_refvalue());

                Ok(Accept::Skip)
            }

            Op::If(then_else) => {
                if context.pop().borrow().is_true() {
                    then_else.0.run(context)
                } else if let Some(eelse) = &then_else.1 {
                    eelse.run(context)
                } else {
                    Ok(Accept::Next)
                }
            }
        }
    }

    fn resolve(&mut self, usages: &mut Vec<Vec<Op>>) {
        match self {
            Op::Usage(usage) => *self = Self::from_vec(usages[*usage].drain(..).collect()),
            Op::Runable(runable) => runable.resolve(usages),
            Op::If(then_else) => {
                then_else.0.resolve(usages);
                then_else.1.as_mut().map(|eelse| eelse.resolve(usages));
            }
            _ => {}
        }
    }

    fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<(bool, bool)> {
        match self {
            Op::Runable(runable) => runable.finalize(statics, stack),
            Op::CallStatic(target) => {
                match &*statics[*target].borrow() {
                    Value::Parselet(parselet) => {
                        if stack.len() > 0 {
                            if let Ok(mut parselet) = parselet.try_borrow_mut() {
                                if !parselet.consuming {
                                    return None;
                                }

                                stack.push((*target, parselet.nullable));
                                let ret = parselet.body.finalize(statics, stack);
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
                                        return Some((i == 0, stack[i].1));
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
                            Some((false, object.is_nullable()))
                        } else {
                            None
                        }
                    }
                }
            }

            Op::If(then_else) => {
                let then = then_else.0.finalize(statics, stack);

                if let Some(eelse) = &mut then_else.1 {
                    if let Some((else_leftrec, else_nullable)) = eelse.finalize(statics, stack) {
                        if let Some((then_leftrec, then_nullable)) = then {
                            Some((then_leftrec || else_leftrec, then_nullable || else_nullable))
                        } else {
                            Some((else_leftrec, else_nullable))
                        }
                    } else {
                        then
                    }
                } else {
                    then
                }
            }

            _ => None,
        }
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Runable(p) => write!(f, "{}", p),
            op => write!(f, "Op {:?}", op),
        }
    }
}

// --- Rust --------------------------------------------------------------------

/*
/** This is allows to run any Rust code in position as Parsable. */
pub struct Rust(pub fn(&mut Context) -> Result<Accept, Reject>);

impl Parsable for Rust {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        self.0(context)
    }
}

impl std::fmt::Debug for Rust {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{rust-function}}")
    }
}

impl std::fmt::Display for Rust {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{rust-function}}")
    }
}
*/
