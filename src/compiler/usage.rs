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
    pub(super) fn try_resolve(&mut self, compiler: &mut Compiler) -> Option<Vec<Op>> {
        match self {
            Usage::Load { name, offset: _ } => {
                if let Some(value) = compiler.get_constant(&name) {
                    return Some(vec![Op::LoadStatic(compiler.define_static(value))]);
                } else if let Some(addr) = compiler.get_local(&name) {
                    return Some(vec![Op::LoadFast(addr)]);
                } else if let Some(addr) = compiler.get_global(&name) {
                    return Some(vec![Op::LoadGlobal(addr)]);
                }
            }

            Usage::CallOrCopy { name, offset: _ } => {
                if let Some(value) = compiler.get_constant(&name) {
                    if value.borrow().is_callable(false) {
                        return Some(vec![Op::CallStatic(compiler.define_static(value))]);
                    } else {
                        return Some(vec![Op::LoadStatic(compiler.define_static(value))]);
                    }
                } else if let Some(addr) = compiler.get_local(&name) {
                    return Some(vec![Op::LoadFast(addr), Op::CallOrCopy]);
                } else if let Some(addr) = compiler.get_global(&name) {
                    return Some(vec![Op::LoadGlobal(addr), Op::CallOrCopy]);
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
                    if value.borrow().is_callable(*args > 0 || *nargs > 0) {
                        let addr = compiler.define_static(value);

                        if *args == 0 && *nargs == 0 {
                            return Some(vec![Op::CallStatic(addr)]);
                        } else if *args > 0 && *nargs == 0 {
                            return Some(vec![Op::CallStaticArg(Box::new((addr, *args)))]);
                        }

                        return Some(vec![Op::CallStaticArgNamed(Box::new((addr, *args)))]);
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
                        return Some(vec![Op::LoadFast(addr), Op::Call]);
                    } else if *args > 0 && *nargs == 0 {
                        return Some(vec![Op::LoadFast(addr), Op::CallArg(*args)]);
                    }

                    return Some(vec![Op::LoadFast(addr), Op::CallArgNamed(*args)]);
                } else if let Some(addr) = compiler.get_global(&name) {
                    if *args == 0 && *nargs == 0 {
                        return Some(vec![Op::LoadGlobal(addr), Op::Call]);
                    } else if *args > 0 && *nargs == 0 {
                        return Some(vec![Op::LoadGlobal(addr), Op::CallArg(*args)]);
                    }

                    return Some(vec![Op::LoadGlobal(addr), Op::CallArgNamed(*args)]);
                }
            }

            Usage::Error(_) => {}
        }

        None
    }

    pub fn resolve_or_dispose(mut self, compiler: &mut Compiler) -> Vec<Op> {
        if let Some(res) = self.try_resolve(compiler) {
            res
        } else {
            compiler.usages.push(Err(self));
            vec![Op::Usage(compiler.usages.len() - 1)]
        }
    }
}
