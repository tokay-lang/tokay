//! Intermediate value representation
use super::*;
use crate::compiler::Compiler;
use crate::reader::Offset;
use crate::value::{Object, RefValue, Value};
use crate::Error;
use indexmap::IndexMap;
use num::ToPrimitive;
use std::cell::RefCell;
use std::rc::Rc;

/** Intermediate value

Intermediate values are values that result during the compile process based on current information
from the syntax tree and symbol table information.

These can be memory locations of variables, static values, functions or values whose definition is
still pending.
*/

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::compiler) enum ImlValue {
    Void,
    Shared(Rc<RefCell<ImlValue>>),

    // Resolved
    Value(RefValue), // Compile-time static value
    Local(usize),    // Runtime local variable
    Global(usize),   // Runtime global variable
    Parselet {
        // Parselet
        parselet: ImlSharedParselet,           // Intermediate definition
        constants: IndexMap<String, ImlValue>, // Generic signature with default configuration
    },

    // Definitely unresolved
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
            Self::Name { name, generic, .. } if !*generic => {
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
            _ => false,
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
            Self::Parselet { parselet, .. } => {
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
            Self::Parselet { parselet, .. } => parselet.borrow().consuming,
            _ => false,
        }
    }

    /// Compile a resolved intermediate value into VM code
    pub fn compile(
        &self,
        program: &mut ImlProgram,
        offset: &Option<Offset>,
        call: Option<Option<(usize, bool)>>,
        ops: &mut Vec<Op>,
    ) {
        if let Some(offset) = offset {
            ops.push(Op::Offset(Box::new(*offset)));
        }

        match self {
            ImlValue::Shared(value) => return value.borrow().compile(program, offset, call, ops),
            ImlValue::Value(value) => {
                if call.is_none() {
                    match &*value.borrow() {
                        // Some frequently used values have built-in push operations
                        Value::Void => return ops.push(Op::PushVoid),
                        Value::Null => return ops.push(Op::PushNull),
                        Value::True => return ops.push(Op::PushTrue),
                        Value::False => return ops.push(Op::PushFalse),
                        Value::Int(i) => match i.to_i64() {
                            Some(0) => return ops.push(Op::Push0),
                            Some(1) => return ops.push(Op::Push1),
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
            ImlValue::Local(addr) => ops.push(Op::LoadFast(*addr)),
            ImlValue::Global(addr) => ops.push(Op::LoadGlobal(*addr)),
            ImlValue::Name { name, .. } => {
                program.errors.push(Error::new(
                    offset.clone(),
                    if call.is_some() {
                        format!("Call to unresolved symbol '{}'", name)
                    } else {
                        format!("Use of unresolved symbol '{}'", name)
                    },
                ));

                return;
            }
            ImlValue::Parselet {
                parselet,
                constants,
            } => {
                // When value is a parselet, check for accepted constant configuration
                let parselet = parselet.borrow();
                let mut required = Vec::new();

                for (name, default) in constants {
                    if matches!(default, ImlValue::Void) {
                        required.push(name.to_string());
                    }
                }

                if !required.is_empty() {
                    program.errors.push(Error::new(
                        offset.clone(),
                        format!(
                            "Call to '{}' requires generic argument {}",
                            self,
                            required.join(", ")
                        ),
                    ));

                    return;
                }
            }
            _ => todo!(),
        }

        // Try to register value as static
        if let Ok(idx) = program.register(self) {
            match call {
                // Load
                None => ops.push(Op::LoadStatic(idx)),
                // Call or load
                Some(None) => {
                    if self.is_callable(true) {
                        ops.push(Op::CallStatic(idx));
                    } else {
                        ops.push(Op::LoadStatic(idx));
                    }
                }
                // Call (qualified)
                Some(Some((0, false))) => ops.push(Op::CallStatic(idx)),
                Some(Some((args, false))) => ops.push(Op::CallStaticArg(Box::new((idx, args)))),
                Some(Some((args, true))) => ops.push(Op::CallStaticArgNamed(Box::new((idx, args)))),
            }
        } else {
            match call {
                // Load (already done previously)
                None => {}
                // Call or load
                Some(None) => ops.push(Op::CallOrCopy),
                // Call (qualified)
                Some(Some((0, false))) => ops.push(Op::Call),
                Some(Some((args, false))) => ops.push(Op::CallArg(args)),
                Some(Some((args, true))) => ops.push(Op::CallArgNamed(args)),
            }
        }
    }
}

impl std::fmt::Display for ImlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Void => write!(f, "void"),
            Self::Shared(value) => value.borrow().fmt(f),
            Self::Value(value) => write!(f, "{}", value.repr()),
            Self::Parselet {
                parselet,
                constants,
            } => {
                write!(f, "{}", parselet)?;

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

                Ok(())
            }
            Self::Name { name, .. } => write!(f, "{}", name),
            _ => todo!(),
        }
    }
}

/*
impl std::fmt::Display for ImlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Void => write!(f, "void"),
            Self::Unknown(name) | Self::Undefined(name) => write!(f, "{}", name),
            Self::Value(value) => write!(f, "{}", value.repr()),
            Self::Parselet {
                parselet,
                constants,
            } => {
                write!(
                    f,
                    "{}",
                    parselet
                        .borrow()
                        .name
                        .as_deref()
                        .unwrap_or("<anonymous parselet>")
                )?;

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

                Ok(())
            }
            Self::Local(addr) => write!(f, "local@{}", addr),
            Self::Global(addr) => write!(f, "global@{}", addr),
            Self::Symbol {
                name,
                gen_by_seq,
                gen_by_name,
            } => {
                write!(f, "{}", target)?;

                let mut first = true;

                for item in gen_by_seq {
                    write!(f, "{}{}", if !first { ", " } else { "<" }, item)?;
                    first = false;
                }

                for (name, item) in gen_by_name.iter() {
                    write!(f, "{}{}:{}", if !first { ", " } else { "<" }, name, item)?;
                    first = false;
                }

                if !first {
                    write!(f, ">")?;
                }

                Ok(())
            }
        }
    }
}
*/

impl std::hash::Hash for ImlValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Value(v) => {
                state.write_u8('v' as u8);
                v.hash(state)
            }
            Self::Parselet {
                parselet,
                constants,
            } => {
                state.write_u8('p' as u8);
                parselet.borrow().hash(state);
                constants.iter().collect::<Vec<_>>().hash(state);
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
