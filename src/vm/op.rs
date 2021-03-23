use super::*;
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
    Usage(usize), // (yet) unresolved usage

    // Parsing constructs
    Empty, // The empty word

    Scanable(Box<dyn Scanable>), // Scanable item
    Runable(Box<dyn Runable>),   // Runable item

    // Call
    TryCall,
    Call,
    CallArg(usize),
    CallArgNamed(usize),
    CallStatic(usize),
    CallStaticArg(Box<(usize, usize)>),
    CallStaticArgNamed(Box<(usize, usize)>),

    // Interrupts
    Skip,
    LoadAccept,
    Reject,

    // Constants
    LoadStatic(usize),
    PushTrue,
    PushFalse,
    PushVoid,

    // Variables & Values
    LoadGlobal(usize),
    LoadFast(usize),
    StoreGlobal(usize),
    StoreFast(usize),
    LoadFastCapture(usize),
    LoadCapture,
    StoreFastCapture(usize),
    StoreCapture,
    //MakeList(usize),
    MakeDict(usize),

    // Operations
    Add,
    Sub,
    Div,
    Mul,
    Not,

    // Flow
    If(Box<(Op, Option<Op>)>),

    // Debug
    Offset(Box<Offset>),
}

impl Op {
    pub fn from_vec(ops: Vec<Op>) -> Self {
        match ops.len() {
            0 => Op::Nop,
            1 => ops.into_iter().next().unwrap(),
            _ => Sequence::new(ops.into_iter().map(|item| (item, None)).collect()),
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

    /*
        Utility function to replace an Op during tranformation
        either by another Op or by a sequence.
    */
    pub(super) fn replace_usage(&mut self, usages: &mut Vec<Vec<Op>>) {
        if let Op::Usage(usage) = self {
            *self = Self::from_vec(usages[*usage].drain(..).collect())
        }
    }
}

impl Runable for Op {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        //println!("RUN {:?}", self);

        match self {
            Op::Nop => Ok(Accept::Next),

            Op::Usage(_) => panic!(
                "{:?} can't be run; Trying to run an unresolved program?",
                self
            ),

            Op::Empty => Ok(Accept::Push(Capture::Empty)),

            Op::Scanable(scanable) => scanable.scan(&mut context.runtime.reader),
            Op::Runable(runable) => runable.run(context),

            Op::TryCall => {
                let value = context.pop();
                if value.borrow().is_callable() {
                    value.borrow().call(context, 0, None)
                } else {
                    Ok(Accept::Push(Capture::Value(value.clone(), 10)))
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

            Op::Skip => Err(Reject::Skip),

            Op::LoadAccept => {
                let value = context.pop();
                Ok(Accept::Return(Some(value.clone())))
            }

            /*
            Op::Accept(value) => {
                Ok(Accept::Return(value.clone()))
            },

            Op::Repeat(value) => {
                Ok(Accept::Repeat(value.clone()))
            },
            */
            Op::Reject => Err(Reject::Return),

            Op::LoadStatic(addr) => Ok(Accept::Push(Capture::Value(
                context.runtime.program.statics[*addr].clone(),
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
            Op::PushVoid => Ok(Accept::Push(Capture::Value(
                Value::Void.into_refvalue(),
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

            Op::StoreGlobal(addr) => {
                // todo: bounds checking?
                context.runtime.stack[*addr] = Capture::Value(context.pop(), 10);

                Ok(Accept::Next)
            }

            Op::StoreFast(addr) => {
                // todo: bounds checking?
                context.runtime.stack[context.stack_start + *addr] =
                    Capture::Value(context.pop(), 10);

                Ok(Accept::Next)
            }

            Op::LoadFastCapture(index) => {
                let value = context
                    .get_capture(*index)
                    .unwrap_or(Value::Void.into_refvalue());

                Ok(Accept::Push(Capture::Value(value, 10)))
            }

            Op::LoadCapture => {
                let index = context.pop();
                let index = index.borrow();

                match *index {
                    Value::Addr(_) | Value::Integer(_) | Value::Float(_) => {
                        Op::LoadFastCapture(index.to_addr()).run(context)
                    }

                    _ => {
                        unimplemented!("//todo")
                    }
                }
            }

            Op::StoreFastCapture(index) => {
                let value = context.pop();

                context.set_capture(*index, value);
                Ok(Accept::Next)
            }

            Op::StoreCapture => {
                let index = context.pop();
                let index = index.borrow();

                match *index {
                    Value::Addr(_) | Value::Integer(_) | Value::Float(_) => {
                        Op::StoreFastCapture(index.to_addr()).run(context)
                    }

                    _ => {
                        unimplemented!("//todo")
                    }
                }
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

            Op::Add | Op::Sub | Op::Div | Op::Mul => {
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
                    Op::Div => (&*a.borrow() / &*b.borrow()).into_refvalue(),
                    Op::Mul => (&*a.borrow() * &*b.borrow()).into_refvalue(),
                    _ => unimplemented!("Unimplemented operator"),
                };

                Ok(Accept::Push(Capture::Value(c, 10)))
            }

            Op::Not => {
                if context.pop().borrow().is_true() {
                    Ok(Accept::Push(Capture::Value(
                        Value::False.into_refvalue(),
                        10,
                    )))
                } else {
                    Ok(Accept::Push(Capture::Value(
                        Value::True.into_refvalue(),
                        10,
                    )))
                }
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

            Op::Offset(offset) => {
                context.source_offset = Some(**offset);
                Ok(Accept::Skip)
            }
        }
    }

    fn finalize(
        &mut self,
        usages: &mut Vec<Vec<Op>>,
        statics: &Vec<RefValue>,
        leftrec: &mut bool,
        nullable: &mut bool,
    ) {
        match self {
            Op::Scanable(_) => {
                *nullable = false;
            }

            Op::Runable(runable) => {
                runable.finalize(usages, statics, leftrec, nullable);
            }

            Op::Usage(_) => self.replace_usage(usages),

            Op::CallStatic(addr) => {
                if statics.len() == 0 {
                    return;
                }

                if let Value::Parselet(parselet) = &*statics[*addr].borrow() {
                    if let Ok(mut parselet) = parselet.try_borrow_mut() {
                        let mut call_leftrec = parselet.leftrec;
                        let mut call_nullable = parselet.nullable;

                        parselet.body.finalize(
                            usages,
                            statics,
                            &mut call_leftrec,
                            &mut call_nullable,
                        );

                        parselet.leftrec = call_leftrec;
                        parselet.nullable = call_nullable;

                        *nullable = parselet.nullable;
                    } else {
                        *leftrec = true;
                    }
                }
            }

            Op::If(then_else) => {
                then_else.0.finalize(usages, statics, leftrec, nullable);

                if let Some(eelse) = &mut then_else.1 {
                    eelse.finalize(usages, statics, leftrec, nullable);
                }
            }

            _ => {}
        }
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Scanable(s) => write!(f, "{}", s),
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
