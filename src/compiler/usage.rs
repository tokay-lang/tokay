use super::*;
use crate::reader::Offset;
use crate::vm::*;

/** Unresolved symbols and calls */
#[derive(Debug)]
pub enum Usage {
    Load {
        name: String,
        offset: Option<Offset>,
    },
    TryCall {
        name: String,
        offset: Option<Offset>,
    },
    Call {
        name: String,
        args: usize,
        nargs: usize,
        offset: Option<Offset>,
    },
}

impl Usage {
    pub(super) fn try_resolve(&self, compiler: &mut Compiler) -> Option<Vec<Op>> {
        match self {
            Usage::Load { name, offset: _ } => {
                if let Some(addr) = compiler.get_constant(&name) {
                    return Some(vec![Op::LoadStatic(addr)]);
                } else if let Some(addr) = compiler.get_local(&name) {
                    return Some(vec![Op::LoadFast(addr)]);
                } else if let Some(addr) = compiler.get_global(&name) {
                    return Some(vec![Op::LoadGlobal(addr)]);
                }
            }

            Usage::TryCall { name, offset: _ } => {
                if let Some(addr) = compiler.get_constant(&name) {
                    let statics = compiler.statics.borrow();

                    // fixme: This should check if the static is callable
                    //        without parameters!
                    if statics[addr].borrow().is_callable(0, 0) {
                        return Some(vec![Op::CallStatic(addr)]);
                    } else {
                        return Some(vec![Op::LoadStatic(addr)]);
                    }
                } else if let Some(addr) = compiler.get_local(&name) {
                    return Some(vec![Op::LoadFast(addr), Op::TryCall]);
                } else if let Some(addr) = compiler.get_global(&name) {
                    return Some(vec![Op::LoadGlobal(addr), Op::TryCall]);
                }
            }

            Usage::Call {
                name,
                args,
                nargs,
                offset: _,
            } => {
                // Resolve constants
                if let Some(addr) = compiler.get_constant(&name) {
                    let statics = compiler.statics.borrow();

                    // fixme: This should check if the static is callable
                    //        without parameters!
                    if statics[addr].borrow().is_callable(*args, *nargs) {
                        if *args == 0 && *nargs == 0 {
                            return Some(vec![Op::CallStatic(addr)]);
                        } else if *args > 0 && *nargs == 0 {
                            return Some(vec![Op::CallStaticArg(Box::new((addr, *args)))]);
                        }

                        return Some(vec![
                            Op::MakeDict(*nargs),
                            Op::CallStaticArgNamed(Box::new((addr, *args))),
                        ]);
                    }
                } else if let Some(addr) = compiler.get_local(&name) {
                    if *args == 0 && *nargs == 0 {
                        return Some(vec![Op::LoadFast(addr), Op::Call]);
                    } else if *args > 0 && *nargs == 0 {
                        return Some(vec![Op::LoadFast(addr), Op::CallArg(*args)]);
                    }

                    return Some(vec![
                        Op::MakeDict(*nargs),
                        Op::LoadFast(addr),
                        Op::CallArgNamed(*args),
                    ]);
                } else if let Some(addr) = compiler.get_global(&name) {
                    if *args == 0 && *nargs == 0 {
                        return Some(vec![Op::LoadGlobal(addr), Op::Call]);
                    } else if *args > 0 && *nargs == 0 {
                        return Some(vec![Op::LoadGlobal(addr), Op::CallArg(*args)]);
                    }

                    return Some(vec![
                        Op::MakeDict(*nargs),
                        Op::LoadGlobal(addr),
                        Op::CallArgNamed(*args),
                    ]);
                }
            }
        }

        None
    }

    pub fn resolve_or_dispose(self, compiler: &mut Compiler) -> Vec<Op> {
        if let Some(res) = self.try_resolve(compiler) {
            //println!("Resolved {:?} into {:?}", self, res);
            res
        } else {
            compiler.usages.push(Err(self));
            vec![Op::Usage(compiler.usages.len() - 1)]
        }
    }
}
