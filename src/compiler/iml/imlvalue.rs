//! Intermediate value representation
use super::*;
use crate::compiler::Compiler;
use crate::reader::Offset;
use crate::utils;
use crate::value::{Object, RefValue, Value};
use crate::Error;
use indexmap::IndexMap;
use num::ToPrimitive;
use std::cell::RefCell;
use std::rc::Rc;

/** Intermediate value

Intermediate values are value descriptors that result during the compile process based on current
information from the syntax tree and symbol table information..

These can be memory locations of variables, static values, parselets or values whose definition is
still pending. As some intermediate values consist of other intermediate values, they are being
modified and resolved during the compilation process.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::compiler) enum ImlValue {
    Unset,                             // Unset
    Unresolved(Rc<RefCell<ImlValue>>), // Unresolved ImlValues are shared
    Value(RefValue),                   // Static value
    Parselet(ImlParselet),             // Parselet
    This(bool),                        // self-reference to function (false) or parselet (true)
    Variable {
        // Resolved variable
        offset: Option<Offset>, // Source offset
        name: String,           // Name
        is_global: bool,        // Global
        addr: usize,            // Address
    },
    Generic {
        // Known generic placeholder
        offset: Option<Offset>, // Source offset
        name: String,           // Identifier
    },
    Name {
        // Unresolved name
        offset: Option<Offset>, // Source offset
        name: String,           // Identifier
    },
    Instance {
        // Unresolved parselet instance definition
        offset: Option<Offset>,                              // Source offset
        target: Box<ImlValue>,                               // Instance target
        args: Vec<(Option<Offset>, ImlValue)>,               // Sequential generic args
        nargs: IndexMap<String, (Option<Offset>, ImlValue)>, // Named generic args
        severity: Option<u8>,                                // optional desired severity
        is_generated: bool,
    },
}

impl ImlValue {
    /**
    Helper function, which creates an instance definition from self,
    by turning self into name<self>.

    This is used internally to implement `Kle<P>` from `P*` syntax
    during the AST traversal.
    */
    pub fn into_generic(self, name: &str, severity: Option<u8>, offset: Option<Offset>) -> Self {
        Self::Instance {
            offset: None,
            target: Box::new(ImlValue::Name {
                offset: None,
                name: name.to_string(),
            }),
            args: vec![(offset, self)],
            nargs: IndexMap::new(),
            severity,
            is_generated: true,
        }
    }

    /// Try to resolve immediatelly, otherwise push shared reference to compiler's unresolved ImlValue.
    pub fn try_resolve(mut self, compiler: &mut Compiler) -> Self {
        if self.resolve(compiler) {
            return self;
        }

        let shared = Self::Unresolved(Rc::new(RefCell::new(self)));
        compiler.scopes[0].usages.push(shared.clone());
        shared
    }

    /**
    In-place resolve unresolved ImlValue.

    - ImlValue::Name are being resolved by the compiler's symbol table
    - ImlValue::Instance are being recursively resolved to produce an ImlValue::Parselet

    Returns true in case the provided value is (already) resolved.
    */
    pub fn resolve(&mut self, compiler: &mut Compiler) -> bool {
        let resolve = match self {
            Self::Unresolved(value) => match value.try_borrow_mut() {
                Ok(mut value) => {
                    if value.resolve(compiler) {
                        Some(value.clone())
                    } else {
                        None
                    }
                }
                Err(_) => todo!("Recursive resolve() impossible by design, see bug #127"),
            },
            Self::Name { offset, name, .. } => compiler.get(offset.clone(), &name),
            Self::Instance {
                offset,
                target,
                args,
                nargs,
                severity,
                is_generated,
            } => {
                let mut is_resolved = target.resolve(compiler);

                // Resolve sequential generic args
                for arg in args.iter_mut() {
                    if !arg.1.resolve(compiler) {
                        is_resolved = false;
                    }
                }

                // Resolve named generic args
                for narg in nargs.values_mut() {
                    if !narg.1.resolve(compiler) {
                        is_resolved = false;
                    }
                }

                // When all instance members are resolved, try to turn the instance definition into a parselet
                if is_resolved {
                    match &**target {
                        ImlValue::Parselet(parselet) => {
                            let parselet = parselet.borrow();
                            let mut generics = IndexMap::new();

                            for (name, default) in parselet.generics.iter() {
                                // Take arguments by sequence first
                                let arg = if !args.is_empty() {
                                    args.remove(0)
                                }
                                // Otherwise, take named arguments by sequence
                                else if let Some(narg) = nargs.shift_remove(name) {
                                    narg
                                }
                                // Otherwise, use default
                                else {
                                    (*offset, default.clone())
                                };

                                // Check integrity of constant names
                                if let Self::Unset = arg.1 {
                                    compiler.errors.push(Error::new(
                                        arg.0,
                                        format!("Expecting argument for generic '{}'", name),
                                    ));
                                } else if arg.1.is_consuming() {
                                    if !utils::identifier_is_consumable(name) {
                                        compiler.errors.push(Error::new(
                                            arg.0,
                                            format!(
                                                "Cannot assign consumable {} to non-consumable generic '{}'",
                                                arg.1, name
                                            )
                                        ));
                                    }
                                } else if utils::identifier_is_consumable(name) {
                                    compiler.errors.push(Error::new(
                                        arg.0,
                                        format!(
                                            "Cannot assign non-consumable {} to consumable generic {} of {}",
                                            arg.1, name, parselet
                                        )
                                    ));
                                }

                                generics.insert(name.clone(), arg.1);
                            }

                            // Report any errors for unconsumed generic arguments.
                            if !args.is_empty() {
                                compiler.errors.push(Error::new(
                                    args[0].0, // report first parameter
                                    format!(
                                        "{} got too many generic arguments ({} in total, expected {})",
                                        target,
                                        generics.len() + args.len(),
                                        generics.len()
                                    ),
                                ));
                            }

                            for (name, (offset, _)) in nargs {
                                if generics.get(name).is_some() {
                                    compiler.errors.push(Error::new(
                                        *offset,
                                        format!(
                                            "{} already got generic argument '{}'",
                                            target, name
                                        ),
                                    ));
                                } else {
                                    compiler.errors.push(Error::new(
                                        *offset,
                                        format!(
                                            "{} does not accept generic argument named '{}'",
                                            target, name
                                        ),
                                    ));
                                }
                            }

                            // Make a parselet instance from the instance definition;
                            // This can be the final parselet instance, but constants
                            // might contain generic references as well, which are being
                            // resolved during further compilation and derivation.
                            let instance = ImlValue::from(ImlParseletInstance {
                                model: parselet.model.clone(),
                                generics,
                                offset: parselet.offset.clone(),
                                name: parselet.name.clone(),
                                severity: severity.unwrap_or(parselet.severity),
                                is_generated: *is_generated,
                            });

                            Some(instance)
                        }
                        target => {
                            compiler.errors.push(Error::new(
                                *offset,
                                format!("Cannot create instance from '{}'", target),
                            ));
                            return false;
                        }
                    }
                } else {
                    return false;
                }
            }
            _ => return true, // anything else is considered as resolved
        };

        if let Some(resolve) = resolve {
            *self = resolve;
            return true;
        }

        false
    }

    /// Conert ImlValue into RefValue
    pub fn unwrap(self) -> RefValue {
        match self {
            Self::Value(value) => value,
            _ => unreachable!("{:?} cannot be unwrapped", self),
        }
    }

    /// Check whether intermediate value represents callable,
    /// and when its callable if with or without arguments.
    pub fn is_callable(&self, without_arguments: bool) -> bool {
        match self {
            Self::Unresolved(value) => value.borrow().is_callable(without_arguments),
            Self::This(_) => true, // fixme?
            Self::Value(value) => value.is_callable(without_arguments),
            Self::Parselet(parselet) => {
                let parselet = parselet.borrow();
                let parselet = parselet.model.borrow();

                if without_arguments {
                    parselet.signature.len() == 0
                        || parselet
                            .signature
                            .iter()
                            .all(|arg| !matches!(arg.1, Self::Unset))
                } else {
                    true
                }
            }
            Self::Instance { target, .. } => target.is_callable(without_arguments),
            _ => false,
        }
    }

    /// Check whether intermediate value represents consuming
    pub fn is_consuming(&self) -> bool {
        match self {
            Self::Unresolved(value) => value.borrow().is_consuming(),
            Self::This(consuming) => *consuming,
            Self::Value(value) => value.is_consuming(),
            Self::Parselet(parselet) => parselet.borrow().model.borrow().is_consuming,
            Self::Name { name, .. } | Self::Generic { name, .. } => {
                crate::utils::identifier_is_consumable(name)
            }
            Self::Instance { target, .. } => target.is_consuming(),
            _ => false,
        }
    }

    /** Compile a resolved intermediate value into VM code or register it as a static.

    The function will panic when the value is not resolved.
    */
    pub fn compile(
        &self,
        program: &mut ImlProgram,
        current: (&ImlParselet, usize),
        offset: &Option<Offset>,
        call: Option<Option<(usize, bool)>>,
        ops: &mut Vec<Op>,
    ) {
        if let Some(offset) = offset {
            ops.push(Op::Offset(Box::new(*offset)));
        }

        // Remember current ops start
        let start = ops.len();

        match self {
            ImlValue::Unset => return,
            ImlValue::Unresolved(value) => {
                return value.borrow().compile(program, current, offset, call, ops)
            }
            ImlValue::Value(value) => match &*value.borrow() {
                Value::Void => ops.push(Op::PushVoid),
                Value::Null => ops.push(Op::PushNull),
                Value::True => ops.push(Op::PushTrue),
                Value::False => ops.push(Op::PushFalse),
                Value::Int(i) => match i.to_i32() {
                    Some(0) => ops.push(Op::Push0),
                    Some(1) => ops.push(Op::Push1),
                    _ => {}
                },
                _ => {}
            },
            ImlValue::Variable {
                addr, is_global, ..
            } => {
                if *is_global {
                    ops.push(Op::LoadGlobal(*addr))
                } else {
                    ops.push(Op::LoadFast(*addr))
                }
            }
            ImlValue::Generic { name, .. } => {
                return current.0.borrow().generics[name]
                    .compile(program, current, offset, call, ops)
            }
            ImlValue::This(_) | ImlValue::Parselet(_) => {}
            _ => unreachable!("{}", self),
        }

        // Check if something has been pushed before.
        if start == ops.len() {
            let idx = match self {
                ImlValue::This(_) => current.1, // use current index
                ImlValue::Parselet(parselet) => match parselet.derive(current.0) {
                    Ok(parselet) => program.register(&ImlValue::Parselet(parselet)),
                    Err(msg) => {
                        program.errors.push(Error::new(offset.clone(), msg));
                        return;
                    }
                },
                resolved => program.register(resolved),
            };

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
            Self::Unset => write!(f, "unset"),
            Self::Unresolved(value) => value.borrow().fmt(f),
            Self::This(true) => write!(f, "Self"),
            Self::This(false) => write!(f, "self"),
            Self::Value(value) => write!(f, "{}", value.repr()),
            Self::Parselet(parselet) => write!(
                f,
                "{}",
                parselet
                    .borrow()
                    .name
                    .as_deref()
                    .unwrap_or("<anonymous parselet>")
            ),
            Self::Variable {
                name, is_global, ..
            } if *is_global => write!(f, "global '{}'", name),
            Self::Variable { name, .. } => write!(f, "local '{}'", name),
            Self::Name { name, .. } => write!(f, "name '{}'", name),
            Self::Generic { name, .. } => write!(f, "generic '{}'", name),
            Self::Instance {
                target,
                args,
                nargs,
                ..
            } => {
                write!(f, "{}", target)?;

                write!(f, "<")?;
                let mut first = true;

                for arg in args {
                    write!(f, "{}{}", if !first { ", " } else { "" }, arg.1)?;
                    first = false;
                }

                for narg in nargs.keys() {
                    write!(
                        f,
                        "{}{}:{}",
                        if !first { ", " } else { "" },
                        narg,
                        nargs[narg].1
                    )?;
                    first = false;
                }

                write!(f, ">")
            }
        }
    }
}

impl std::hash::Hash for ImlValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Unset => state.write_u8('u' as u8),
            Self::Unresolved(value) => value.borrow().hash(state),
            Self::Value(value) => {
                state.write_u8('v' as u8);
                value.hash(state)
            }
            Self::Parselet(parselet) => {
                state.write_u8('p' as u8);
                parselet.hash(state);
            }
            /*
            Self::This(consumable) => {
                state.write_u8('s' as u8);
                consumable.hash(state);
            }
            */
            other => unreachable!("{} is unhashable", other),
        }
    }
}

impl From<RefValue> for ImlValue {
    fn from(value: RefValue) -> Self {
        Self::Value(value)
    }
}
