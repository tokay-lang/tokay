use super::*;
use crate::value::{RefValue, Value};

/** Intermediate code representation. */

#[derive(Debug)]
pub enum ImlOp {
    Nop,
    Usage(usize),              // (yet) unresolved usage
    Runable(Box<dyn Runable>), // Runable item
    Op(Op),                    // VM Operation
}

impl ImlOp {
    pub fn from_vec(ops: Vec<ImlOp>) -> Self {
        match ops.len() {
            0 => ImlOp::Nop,
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

impl Runable for ImlOp {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        match self {
            ImlOp::Nop => Ok(Accept::Next),
            ImlOp::Usage(_) => panic!(
                "{:?} can't be run; Trying to run an unresolved program?",
                self
            ),
            ImlOp::Runable(runable) => runable.run(context),
            ImlOp::Op(op) => Op::run(&[op], context),
        }
    }

    fn compile(&self) -> Vec<Op> {
        match self {
            ImlOp::Nop => Vec::new(),
            ImlOp::Usage(_) => panic!("Cannot compile Iml::Usage"),
            ImlOp::Runable(r) => r.compile(),
            ImlOp::Op(op) => vec![op.clone()],
        }
    }

    fn resolve(&mut self, usages: &mut Vec<Vec<ImlOp>>) {
        match self {
            ImlOp::Usage(usage) => *self = Self::from_vec(usages[*usage].drain(..).collect()),
            ImlOp::Runable(runable) => runable.resolve(usages),
            _ => {}
        }
    }

    fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<(bool, bool)> {
        match self {
            ImlOp::Runable(runable) => runable.finalize(statics, stack),
            ImlOp::Op(Op::CallStatic(target)) => {
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

            _ => None,
        }
    }
}

impl std::fmt::Display for ImlOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImlOp::Runable(p) => write!(f, "{}", p),
            op => write!(f, "Op {:?}", op),
        }
    }
}

impl From<Op> for ImlOp {
    fn from(op: Op) -> Self {
        ImlOp::Op(op)
    }
}
