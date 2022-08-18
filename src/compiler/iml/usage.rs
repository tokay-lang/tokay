//! Usages are placeholders to symbols which are replaced later by VM code during compilation

use super::*;
use crate::reader::Offset;
use crate::Compiler;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub enum Action {
    Load,   // Load a value
    //LoadAndCopy, // Load a value and duplicate (if duplication is possible, otherwise just load)
    CallOrCopy,  // Load a value and call when callable without parameters, otherwise duplicate
    Call(usize, bool)  // Qualified call with args and nargs.
}

#[derive(Debug)]
pub enum Target {
    Static(ImlValue),   // Compile-time static value
    Global(usize),  // Runtime global value
    Local(usize),  // Runtime local value
}

/** Unresolved symbols and calls */
#[derive(Debug)]
pub struct Usage {
    offset: Option<Offset>,
    action: Action,
    target: Result<Target, String>,
}

impl Usage {
    /// Call unknown value
    pub fn call(compiler: &mut Compiler, offset: Option<Offset>, name: String, args: usize, nargs: bool) -> ImlOp {
        Self{
            offset,
            action: if args == 0 && !nargs { Action::CallOrCopy } else { Action::Call(args, nargs) },
            target: Err(name),
        }.try_resolve(compiler)
    }

    /// Load unknown value
    pub fn load(compiler: &mut Compiler, offset: Option<Offset>, name: String) -> ImlOp {
        Self{
            offset,
            action: Action::Load,
            target: Err(name),
        }.try_resolve(compiler)
    }

    /// Try to resolve immediatelly, otherwise push a ImlOp::Usage placeholder for later resolve.
    fn try_resolve(mut self, compiler: &mut Compiler) -> ImlOp {
        if self.resolve(compiler) {
            return ImlOp::Usage(self);
        }

        let shared = Rc::new(RefCell::new(ImlOp::Usage(self)));
        compiler.usages.push(shared.clone());
        ImlOp::Shared(shared)
    }

    pub fn resolve(&mut self, compiler: &mut Compiler) -> bool {
        match self.target {
            Ok(_) => true,
            Err(name) => {
                if let Some(value) = compiler.get_constant(&name) {
                    // Undetermined usages need to remain untouched.
                    if !matches!(value, ImlValue::Undetermined(_)) {
                        self.target = Ok(Target::Static(value));
                        return true;
                    }
                } else if let Some(addr) = compiler.get_local(&name) {
                    self.target = Ok(Target::Local(addr));
                    return true;
                } else if let Some(addr) = compiler.get_global(&name) {
                    self.target = Ok(Target::Global(addr));
                    return true;
                }

                false
            }
        }

        /*
        let mut ret: Vec<ImlOp> = Vec::new();

        match self {
            Usage::Load { name, offset: _ } => {
                if let Some(value) = compiler.get_constant(&name) {
                    // Undetermined usages need to remain untouched.
                    if !matches!(value, ImlValue::Undetermined(_)) {
                        ret.push(ImlOp::Load(ImlOpValue(value)));
                    }
                } else if let Some(addr) = compiler.get_local(&name) {
                    ret.push(Op::LoadFast(addr).into())
                } else if let Some(addr) = compiler.get_global(&name) {
                    ret.push(Op::LoadGlobal(addr).into())
                }
            }

            Usage::CallOrCopy { name, offset } => {
                if let Some(value) = compiler.get_constant(&name) {
                    // Undetermined usages need to remain untouched.
                    if !matches!(value, ImlValue::Undetermined(_)) {
                        if value.is_callable(true) {
                            ret.push(ImlOp::Call(ImlOpValue(value), 0, false, *offset));
                        } else {
                            ret.push(ImlOp::Load(ImlOpValue(value)));
                        }
                    }
                } else if let Some(addr) = compiler.get_local(&name) {
                    if let Some(offset) = offset {
                        ret.push(Op::Offset(Box::new(*offset)).into());
                    }

                    ret.push(Op::LoadFast(addr).into());
                    ret.push(Op::CallOrCopy.into());
                } else if let Some(addr) = compiler.get_global(&name) {
                    if let Some(offset) = offset {
                        ret.push(Op::Offset(Box::new(*offset)).into());
                    }

                    ret.push(Op::LoadGlobal(addr).into());
                    ret.push(Op::CallOrCopy.into());
                }
            }

            Usage::Call {
                name,
                args,
                nargs,
                offset,
            } => {
                // Resolve constants
                if let Some(value) = compiler.get_constant(&name) {
                    if !matches!(value, ImlValue::Undetermined(_)) {
                        ret.push(ImlOp::Call(ImlOpValue(value), *args, *nargs, *offset));
                    }
                } else if let Some(addr) = compiler.get_local(&name) {
                    if let Some(offset) = offset {
                        ret.push(Op::Offset(Box::new(*offset)).into());
                    }

                    if *args == 0 && *nargs == false {
                        ret.push(Op::LoadFast(addr).into());
                        ret.push(Op::Call.into());
                    } else if *args > 0 && *nargs == false {
                        ret.push(Op::LoadFast(addr).into());
                        ret.push(Op::CallArg(*args).into());
                    } else {
                        ret.push(Op::LoadFast(addr).into());
                        ret.push(Op::CallArgNamed(*args).into());
                    }
                } else if let Some(addr) = compiler.get_global(&name) {
                    if let Some(offset) = offset {
                        ret.push(Op::Offset(Box::new(*offset)).into());
                    }

                    if *args == 0 && *nargs == false {
                        ret.push(Op::LoadGlobal(addr).into());
                        ret.push(Op::Call.into());
                    } else if *args > 0 && *nargs == false {
                        ret.push(Op::LoadGlobal(addr).into());
                        ret.push(Op::CallArg(*args).into());
                    } else {
                        ret.push(Op::LoadGlobal(addr).into());
                        ret.push(Op::CallArgNamed(*args).into());
                    }
                }
            }
        }

        if ret.len() > 0 {
            Some(ImlOp::from(ret))
        } else {
            None
        }
        */
    }

    /// Does this Usage refer to a consumable constant?
    pub fn is_consuming(&self) -> bool {
        match self {
            Usage::CallOrCopy { name, .. } | Usage::Call { name, .. } => {
                crate::utils::identifier_is_consumable(name)
            }
            _ => false,
        }
    }
}
