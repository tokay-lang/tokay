//! Intermediate value representation
use super::*;
use crate::compiler::Compiler;
use crate::reader::Offset;
use crate::value::{Object, RefValue, Value};
use crate::Error;
use num::ToPrimitive;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/** Intermediate value

Intermediate values are values that result during the compile process based on current information
from the syntax tree and symbol table information.

These can be memory locations of variables, static values, functions or values whose definition is
still pending.
*/

#[derive(Clone, PartialEq, Eq)]
pub(in crate::compiler) enum ImlValue {
    Void,
    Shared(Rc<RefCell<ImlValue>>),

    // Resolved
    Value(RefValue),                    // Compile-time static value
    Local(usize),                       // Runtime local variable
    Global(usize),                      // Runtime global variable
    Parselet(Rc<RefCell<ImlParselet>>), // Parselet

    // Unresolved
    Name {
        // Unresolved name
        offset: Option<Offset>, // Source offset
        generic: bool,          // Generic name, to be resolved during compilation
        name: String,           // Identifier
    },
    Instance {
        // Parselet instance
        offset: Option<Offset>, // Source offset
        target: Box<ImlValue>,  // Instance target
        config: Vec<(Option<Offset>, Option<String>, ImlValue)>, // Constant configuration
    },
}

impl ImlValue {
    /// Try to resolve immediatelly, otherwise push shared reference to compiler's unresolved ImlValue.
    pub fn try_resolve(mut self, compiler: &mut Compiler) -> Self {
        if self.resolve(compiler) {
            return self;
        }

        let shared = Self::Shared(Rc::new(RefCell::new(self)));
        compiler.usages.push(shared.clone());
        shared
    }

    /// Resolve unresolved ImlValue. Returns true in case the provided value is (already) resolved.
    pub fn resolve(&mut self, compiler: &mut Compiler) -> bool {
        match self {
            Self::Shared(value) => value.borrow_mut().resolve(compiler),
            Self::Name { name, .. } => {
                if let Some(value) = compiler.get(&name) {
                    *self = value;
                    true
                } else {
                    false
                }
            }
            /*
            Self::Instance {
                target,
                ..
            } if matches!(target, ImlValue::Name(_)) => {
                // Try to resolve target
                if target.resolve(compiler) {
                    // On success, try to resolve the entire instance
                    return self.resolve(compiler);
                }
            }
            Self::Instance {
                target:
                    ImlValue::Parselet {
                        parselet,
                        constants,
                    },
                config,
                offset,
            } => {
                todo!();
            }
            */
            _ => true,
        }
    }

    pub fn into_refvalue(self) -> RefValue {
        match self {
            Self::Value(value) => value,
            _ => unreachable!("{:?} cannot be unwrapped", self),
        }
    }

    /// Check whether intermediate value represents callable,
    /// and when its callable if with or without arguments.
    pub fn is_callable(&self, without_arguments: bool) -> bool {
        match self {
            Self::Shared(value) => value.borrow().is_callable(without_arguments),
            Self::Value(value) => value.is_callable(without_arguments),
            Self::Parselet(parselet) => {
                let parselet = parselet.borrow();

                if without_arguments {
                    parselet.signature.len() == 0
                        || parselet
                            .signature
                            .iter()
                            .all(|arg| !matches!(arg.1, Self::Void))
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    /// Check whether intermediate value represents consuming
    pub fn is_consuming(&self) -> bool {
        match self {
            Self::Shared(value) => value.borrow().is_consuming(),
            Self::Name { name, .. } => crate::utils::identifier_is_consumable(name),
            Self::Value(value) => value.is_consuming(),
            Self::Parselet(parselet) => parselet.borrow().consuming,
            _ => false,
        }
    }

    // Finalize... this is a work in progress...
    pub fn finalize(
        &self,
        visited: &mut HashSet<usize>,
        configs: &mut HashMap<usize, Consumable>,
    ) -> Option<Consumable> {
        match self {
            ImlValue::Shared(value) => value.borrow().finalize(visited, configs),
            ImlValue::Parselet(parselet) => {
                match parselet.try_borrow() {
                    // In case the parselet cannot be borrowed, it is left-recursive!
                    Err(_) => Some(Consumable {
                        leftrec: true,
                        nullable: false,
                    }),
                    // Otherwise dive into this parselet...
                    Ok(parselet) => {
                        // ... only if it's generally flagged to be consuming.
                        if !parselet.consuming {
                            return None;
                        }

                        let id = parselet.id();

                        if visited.contains(&id) {
                            Some(Consumable {
                                leftrec: false,
                                nullable: configs[&id].nullable,
                            })
                        } else {
                            visited.insert(id);

                            if !configs.contains_key(&id) {
                                configs.insert(
                                    id,
                                    Consumable {
                                        leftrec: false,
                                        nullable: false,
                                    },
                                );
                            }

                            //fixme: Finalize on begin and end as well!
                            let ret = parselet.body.finalize(visited, configs);

                            visited.remove(&id);

                            ret
                        }
                    }
                }
            }
            ImlValue::Value(callee) => {
                if callee.is_consuming() {
                    //println!("{:?} called, which is nullable={:?}", callee, callee.is_nullable());
                    Some(Consumable {
                        leftrec: false,
                        nullable: callee.is_nullable(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /** Generates code for a value load. For several, oftenly used values, there exists a direct operation pendant,
    which makes storing the static value obsolete. Otherwise, *value* will be registered and a static load operation
    is returned. */
    pub fn compile_to_load(&self, linker: &mut Linker) -> Op {
        match self {
            ImlValue::Shared(value) => return value.borrow().compile_to_load(linker),
            ImlValue::Value(value) => match &*value.borrow() {
                Value::Void => return Op::PushVoid,
                Value::Null => return Op::PushNull,
                Value::True => return Op::PushTrue,
                Value::False => return Op::PushFalse,
                Value::Int(i) => match i.to_i64() {
                    Some(0) => return Op::Push0,
                    Some(1) => return Op::Push1,
                    _ => {}
                },
                _ => {}
            },
            ImlValue::Parselet(_) => {}
            ImlValue::Local(addr) => return Op::LoadFast(*addr),
            ImlValue::Global(addr) => return Op::LoadGlobal(*addr),
            ImlValue::Name { name, .. } => {
                linker.errors.push(Error::new(
                    None,
                    format!("Use of unresolved symbol '{}'", name),
                ));

                return Op::Nop;
            }
            _ => todo!(),
        }

        Op::LoadStatic(linker.register(self))
    }

    /** Generates code for a value call. */
    pub fn compile_to_call(&self, linker: &mut Linker, args: Option<(usize, bool)>) -> Vec<Op> {
        let mut ops = Vec::new();

        match self {
            ImlValue::Shared(value) => return value.borrow().compile_to_call(linker, args),
            ImlValue::Local(addr) => ops.push(Op::LoadFast(*addr)),
            ImlValue::Global(addr) => ops.push(Op::LoadGlobal(*addr)),
            ImlValue::Name { name, .. } => {
                linker.errors.push(Error::new(
                    None,
                    format!("Call to unresolved symbol '{}'", name),
                ));
                return ops;
            }
            value => {
                // When value is a parselet, check for accepted constant configuration
                /*
                if let ImlValue::Parselet {
                    parselet: _,
                    constants,
                } = value
                {
                    if !constants.is_empty() {
                        let mut required = Vec::new();

                        for (name, default) in constants {
                            if matches!(default, ImlValue::Void) {
                                required.push(name.to_string());
                            }
                        }

                        if !required.is_empty() {
                            linker.errors.push(Error::new(
                                offset.clone(),
                                format!(
                                    "On call to '{}', missing generic constants for {}",
                                    value,
                                    required.join(", ")
                                ),
                            ));

                            return 0;
                        }
                    }
                }
                */

                let idx = linker.register(value);

                match args {
                    // Qualified call
                    Some((args, nargs)) => {
                        if args == 0 && !nargs {
                            ops.push(Op::CallStatic(idx));
                        } else if args > 0 && !nargs {
                            ops.push(Op::CallStaticArg(Box::new((idx, args))));
                        } else {
                            ops.push(Op::CallStaticArgNamed(Box::new((idx, args))));
                        }
                    }
                    // Call or load
                    None => {
                        if value.is_callable(true) {
                            ops.push(Op::CallStatic(idx));
                        } else {
                            ops.push(Op::LoadStatic(idx));
                        }
                    }
                }

                return ops;
            }
            _ => todo!(),
        }

        match args {
            // Qualified call
            Some((args, nargs)) => {
                if args == 0 && nargs == false {
                    ops.push(Op::Call);
                } else if args > 0 && nargs == false {
                    ops.push(Op::CallArg(args));
                } else {
                    ops.push(Op::CallArgNamed(args));
                }
            }
            // Call or load
            None => ops.push(Op::CallOrCopy),
        }

        ops
    }
}

impl std::fmt::Debug for ImlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Void => write!(f, "void"),
            Self::Shared(value) => value.borrow().fmt(f),
            Self::Value(v) => v.borrow().fmt(f),
            Self::Parselet { .. } => write!(f, "{}", self),
            Self::Local(addr) => write!(f, "local@{}", addr),
            Self::Global(addr) => write!(f, "global@{}", addr),
            Self::Name { name, .. } => write!(f, "{}", name),
            _ => todo!(),
        }
    }
}

impl std::fmt::Display for ImlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Void => write!(f, "void"),
            Self::Shared(value) => value.borrow().fmt(f),
            Self::Value(value) => write!(f, "{}", value.repr()),
            Self::Parselet(parselet) => {
                write!(
                    f,
                    "{}",
                    parselet
                        .borrow()
                        .name
                        .as_deref()
                        .unwrap_or("<anonymous parselet>")
                )?;

                /*
                if !constants.is_empty() {
                    write!(f, "<")?;
                    for (i, (name, value)) in constants.iter().enumerate() {
                        if matches!(value, ImlValue::Void) {
                            write!(f, "{}{}", if i > 0 { ", " } else { "" }, name)?;
                        } else {
                            write!(f, "{}{}:{}", if i > 0 { ", " } else { "" }, name, value)?;
                        }
                    }
                    write!(f, ">")?;
                }
                */

                Ok(())
            }
            Self::Name { name, .. } => write!(f, "{}", name),
            _ => todo!(),
        }
    }
}

impl std::hash::Hash for ImlValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Value(v) => {
                state.write_u8('v' as u8);
                v.hash(state)
            }
            Self::Parselet(parselet) => {
                state.write_u8('p' as u8);
                parselet.borrow().hash(state);
                //constants.iter().collect::<Vec<_>>().hash(state);
            }
            other => unreachable!("{:?} is unhashable", other),
        }
    }
}

impl From<RefValue> for ImlValue {
    fn from(value: RefValue) -> Self {
        Self::Value(value)
    }
}
