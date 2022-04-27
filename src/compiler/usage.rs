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
        let mut ret: Vec<ImlOp> = Vec::new();

        match self {
            Usage::Load { name, offset: _ } => {
                if let Some(value) = compiler.get_constant(&name) {
                    ret.push(Op::LoadStatic(compiler.define_value(value)).into());
                } else if let Some(addr) = compiler.get_local(&name) {
                    ret.push(Op::LoadFast(addr).into())
                } else if let Some(addr) = compiler.get_global(&name) {
                    ret.push(Op::LoadGlobal(addr).into())
                }
            }

            Usage::CallOrCopy { name, offset } => {
                if let Some(value) = compiler.get_constant(&name) {
                    if value.is_callable(false) {
                        if let Some(offset) = offset {
                            ret.push(Op::Offset(Box::new(*offset)).into());
                        }

                        ret.push(Op::CallStatic(compiler.define_value(value)).into());
                    } else {
                        ret.push(Op::LoadStatic(compiler.define_value(value)).into());
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
                    if value.is_callable(*args > 0 || *nargs > 0) {
                        if let Some(offset) = offset {
                            ret.push(Op::Offset(Box::new(*offset)).into());
                        }

                        let addr = compiler.define_value(value);

                        if *args == 0 && *nargs == 0 {
                            ret.push(Op::CallStatic(addr).into());
                        } else if *args > 0 && *nargs == 0 {
                            if let Some(offset) = offset {
                                ret.push(Op::Offset(Box::new(*offset)).into());
                            }

                            ret.push(Op::CallStaticArg(Box::new((addr, *args))).into());
                        } else {
                            ret.push(Op::CallStaticArgNamed(Box::new((addr, *args))).into());
                        }
                    } else if *args == 0 && *nargs == 0 {
                        *self = Usage::Error(Error::new(
                            *offset,
                            format!("{}() expects arguments for call", name),
                        ));
                    } else {
                        *self = Usage::Error(Error::new(
                            *offset,
                            format!(
                                "{}() doesn't accept any arguments ({} given)",
                                name,
                                *args + *nargs
                            ),
                        ));
                    }
                } else if let Some(addr) = compiler.get_local(&name) {
                    if let Some(offset) = offset {
                        ret.push(Op::Offset(Box::new(*offset)).into());
                    }

                    if *args == 0 && *nargs == 0 {
                        ret.push(Op::LoadFast(addr).into());
                        ret.push(Op::Call.into());
                    } else if *args > 0 && *nargs == 0 {
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

                    if *args == 0 && *nargs == 0 {
                        ret.push(Op::LoadGlobal(addr).into());
                        ret.push(Op::Call.into());
                    } else if *args > 0 && *nargs == 0 {
                        ret.push(Op::LoadGlobal(addr).into());
                        ret.push(Op::CallArg(*args).into());
                    } else {
                        ret.push(Op::LoadGlobal(addr).into());
                        ret.push(Op::CallArgNamed(*args).into());
                    }
                }
            }

            Usage::Error(_) => {
                // Just ignore already errored usage.
            }
        }

        if ret.len() > 0 {
            Some(ret)
        } else {
            None
        }
    }

    /// Try to resolve immediatelly, otherwise push a ImlOp::Usage placeholder for later resolve.
    pub fn resolve_or_dispose(mut self, compiler: &mut Compiler) -> Vec<ImlOp> {
        if let Some(res) = self.try_resolve(compiler) {
            res
        } else {
            compiler.usages.push(Err(self));
            vec![ImlOp::Usage(compiler.usages.len() - 1)]
        }
    }
}
