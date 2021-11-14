//! Usages are placeholders to symbols which are replaced later by VM code during compilation

use super::*;
use crate::error::Error;
use crate::reader::Offset;

/** Unresolved symbols and calls */
#[derive(Debug)]
pub enum Usage {
    // Qualified symbol load
    Load {
        name: String,
        offset: Option<Offset>,
    },
    // Either load a symbol or call it when it is callable without parameters.
    CallOrCopy {
        name: String,
        offset: Option<Offset>,
    },
    // Qualified symbol call
    Call {
        name: String,
        args: usize,
        nargs: usize,
        offset: Option<Offset>,
    },
    // Error during resolve
    Error(Error),
}

impl Usage {
    pub fn try_resolve(&mut self, compiler: &mut Compiler) -> Option<Vec<ImlOp>> {
        match self {
            Usage::Load { name, offset: _ } => {
                if let Some(value) = compiler.get_constant(&name) {
                    return Some(vec![Op::LoadStatic(compiler.define_value(value)).into()]);
                } else if let Some(addr) = compiler.get_local(&name) {
                    return Some(vec![Op::LoadFast(addr).into()]);
                } else if let Some(addr) = compiler.get_global(&name) {
                    return Some(vec![Op::LoadGlobal(addr).into()]);
                }
            }

            Usage::CallOrCopy { name, offset: _ } => {
                if let Some(value) = compiler.get_constant(&name) {
                    if value.is_callable(false) {
                        return Some(vec![Op::CallStatic(compiler.define_value(value)).into()]);
                    } else {
                        return Some(vec![Op::LoadStatic(compiler.define_value(value)).into()]);
                    }
                } else if let Some(addr) = compiler.get_local(&name) {
                    return Some(vec![Op::LoadFast(addr).into(), Op::CallOrCopy.into()]);
                } else if let Some(addr) = compiler.get_global(&name) {
                    return Some(vec![Op::LoadGlobal(addr).into(), Op::CallOrCopy.into()]);
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
                    if value.is_callable(*args > 0 || *nargs > 0) {
                        let addr = compiler.define_value(value);

                        if *args == 0 && *nargs == 0 {
                            return Some(vec![Op::CallStatic(addr).into()]);
                        } else if *args > 0 && *nargs == 0 {
                            return Some(vec![Op::CallStaticArg(Box::new((addr, *args))).into()]);
                        }

                        return Some(vec![Op::CallStaticArgNamed(Box::new((addr, *args))).into()]);
                    } else if *args == 0 && *nargs == 0 {
                        *self = Usage::Error(Error::new(
                            *offset,
                            format!("Call to '{}' doesn't accept any arguments", name),
                        ));
                    } else {
                        *self = Usage::Error(Error::new(
                            *offset,
                            format!("'{}' cannot be called without arguments", name),
                        ));
                    }
                } else if let Some(addr) = compiler.get_local(&name) {
                    if *args == 0 && *nargs == 0 {
                        return Some(vec![Op::LoadFast(addr).into(), Op::Call.into()]);
                    } else if *args > 0 && *nargs == 0 {
                        return Some(vec![Op::LoadFast(addr).into(), Op::CallArg(*args).into()]);
                    }

                    return Some(vec![
                        Op::LoadFast(addr).into(),
                        Op::CallArgNamed(*args).into(),
                    ]);
                } else if let Some(addr) = compiler.get_global(&name) {
                    if *args == 0 && *nargs == 0 {
                        return Some(vec![Op::LoadGlobal(addr).into(), Op::Call.into()]);
                    } else if *args > 0 && *nargs == 0 {
                        return Some(vec![Op::LoadGlobal(addr).into(), Op::CallArg(*args).into()]);
                    }

                    return Some(vec![
                        Op::LoadGlobal(addr).into(),
                        Op::CallArgNamed(*args).into(),
                    ]);
                }
            }

            Usage::Error(_) => {}
        }

        None
    }

    pub fn resolve_or_dispose(mut self, compiler: &mut Compiler) -> Vec<ImlOp> {
        if let Some(res) = self.try_resolve(compiler) {
            res
        } else {
            compiler.usages.push(Err(self));
            vec![ImlOp::Usage(compiler.usages.len() - 1)]
        }
    }
}
