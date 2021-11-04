use super::*;
use crate::value::{RefValue, Value};

/** Intermediate code representation. */

#[derive(Debug)]
pub enum ImlOp {
    Nop,
    Usage(usize),                      // (yet) unresolved usage
    Compileable(Box<dyn Compileable>), // Compileable item
    Op(Op),                            // VM Operation
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

impl Compileable for ImlOp {
    fn compile(&self, parselet: &ImlParselet) -> Vec<Op> {
        match self {
            ImlOp::Nop => Vec::new(),
            ImlOp::Usage(_) => panic!("Cannot compile Iml::Usage"),
            ImlOp::Compileable(r) => r.compile(parselet),
            ImlOp::Op(op) => vec![op.clone()],
        }
    }

    fn resolve(&mut self, usages: &mut Vec<Vec<ImlOp>>) {
        match self {
            ImlOp::Usage(usage) => *self = Self::from_vec(usages[*usage].drain(..).collect()),
            ImlOp::Compileable(runable) => runable.resolve(usages),
            _ => {}
        }
    }

    fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<Consumable> {
        match self {
            ImlOp::Compileable(runable) => runable.finalize(statics, stack),
            ImlOp::Op(Op::CallStatic(target)) => {
                match &*statics[*target].borrow() {
                    Value::ImlParselet(parselet) => {
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
                                let ret = parselet.finalize(statics, stack);
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
