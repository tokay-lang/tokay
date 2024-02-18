//! Intermediate value representation
use super::*;
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
    Shared(Rc<RefCell<ImlValue>>), // Shared ImlValues are used for later resolve
    SelfValue,                     // self-reference (value)
    SelfToken,                     // Self-reference (consuming)
    VoidToken,                     // Void (consuming)
    Value(RefValue),               // Static value
    Parselet(ImlParselet),         // Parselet
    Variable {
        // Resolved variable
        offset: Option<Offset>, // Source offset
        name: String,           // Name
        is_global: bool,        // Global
        addr: usize,            // Address
    },
    Generic {
        // Generic placeholder
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
    pub fn into_generic(
        self,
        name: &str,
        scope: &Scope,
        severity: Option<u8>,
        offset: Option<Offset>,
    ) -> Self {
        Self::Instance {
            offset: None,
            target: Box::new(
                ImlValue::Name {
                    offset: None,
                    name: name.to_string(),
                }
                .try_resolve(scope),
            ),
            args: vec![(offset, self)],
            nargs: IndexMap::new(),
            severity,
            is_generated: true,
        }
        .try_resolve(scope)
    }

    /// Checks whether the given intermediate value is resolved or not.
    pub fn is_resolved(&self) -> bool {
        match self {
            Self::Shared(value) => value.borrow().is_resolved(),
            Self::Name { .. } | Self::Instance { .. } => false,
            _ => true,
        }
    }

    /// Returns the value's definition offset, if available
    pub fn offset(&self) -> Option<Offset> {
        match self {
            Self::Shared(value) => value.borrow().offset(),
            Self::Name { offset, .. } | Self::Instance { offset, .. } => offset.clone(),
            _ => None,
        }
    }

    /// Try to resolve immediatelly, otherwise push shared reference to compiler's unresolved ImlValue.
    pub fn try_resolve(self, scope: &Scope) -> Self {
        if self.is_resolved() {
            // TODO: Clean shared chain?
            return self;
        }

        match self.resolve(scope) {
            Some(value) => value,
            None => {
                if matches!(self, Self::Shared(_)) {
                    self
                } else {
                    let shared = Self::Shared(Rc::new(RefCell::new(self)));
                    scope.usages.borrow_mut().push(shared.clone());
                    shared
                }
            }
        }
    }

    /**
    Resolve unresolved ImlValue.

    - ImlValue::Shared are being followed
    - ImlValue::Name are being resolved by the compiler's symbol table
    - ImlValue::Instance are being recursively resolved to produce an ImlValue::Parselet

    Returns Some(value) with the resolved value on success, None otherwise.
    */
    fn resolve(&self, scope: &Scope) -> Option<ImlValue> {
        match self {
            Self::Shared(value) => match value.try_borrow_mut() {
                Ok(mut value) => {
                    if let Some(replace) = value.resolve(scope) {
                        *value = replace.clone();
                        Some(replace)
                    } else {
                        None
                    }
                }
                Err(_) => todo!("Recursive resolve() impossible by design, see bug #127"),
            },
            Self::Name { offset, name, .. } => scope.resolve_name(offset.clone(), &name),
            Self::Instance {
                offset,
                target,
                args,
                nargs,
                severity,
                is_generated,
            } => {
                match target.resolve(scope) {
                    Some(ImlValue::Parselet(parselet)) => {
                        let parselet = parselet.borrow();
                        let mut generics = IndexMap::new();

                        // TODO: This should be improved, clone should not be used
                        let mut args = args.clone();
                        let mut nargs = nargs.clone();
                        let mut resolved = true;
                        let mut errors = Vec::new();

                        for (name, default) in parselet.generics.iter() {
                            // Take arguments by sequence first
                            let arg = if !args.is_empty() {
                                let (offset, mut arg) = args.remove(0);

                                if let Some(value) = arg.resolve(scope) {
                                    arg = value;
                                } else {
                                    resolved = false;
                                    continue;
                                }

                                (offset, Some(arg))
                            }
                            // Otherwise, take named arguments
                            else if let Some(narg) = nargs.shift_remove(name) {
                                let (offset, mut arg) = narg;

                                if let Some(value) = arg.resolve(scope) {
                                    arg = value;
                                } else {
                                    resolved = false;
                                    continue;
                                }

                                (offset, Some(arg))
                            }
                            // Otherwise, use default
                            else {
                                (*offset, default.clone())
                            };

                            // Check integrity of constant names
                            if let (offset, Some(value)) = &arg {
                                if value.is_consuming() {
                                    if !utils::identifier_is_consumable(name) {
                                        errors.push((
                                            *offset,
                                            format!(
                                                "Cannot assign consumable {} to non-consumable generic '{}'",
                                                value, name
                                            )
                                        ));
                                    }
                                } else if utils::identifier_is_consumable(name) {
                                    errors.push((
                                        *offset,
                                        format!(
                                            "Cannot assign non-consumable {} to consumable generic {} of {}",
                                            value, name, parselet
                                        )
                                    ));
                                }
                            } else {
                                errors.push((
                                    arg.0,
                                    format!("Expecting argument for generic '{}'", name),
                                ));
                            }

                            generics.insert(name.clone(), arg.1);
                        }

                        // In case the instance is not fully resolved, don't continue
                        if !resolved {
                            return None;
                        }

                        // In case errors occured during generic argument collection, merge them into the scope
                        for error in errors {
                            scope.error(error.0, error.1);
                        }

                        // Report any errors for unconsumed generic arguments.
                        if !args.is_empty() {
                            scope.error(
                                args[0].0, // report first parameter
                                format!(
                                    "{} got too many generic arguments ({} given, {} expected)",
                                    target,
                                    generics.len() + args.len(),
                                    generics.len()
                                ),
                            );
                        }

                        for (name, (offset, _)) in nargs {
                            if generics.get(&name).is_some() {
                                scope.error(
                                    offset,
                                    format!("{} already got generic argument '{}'", target, name),
                                );
                            } else {
                                scope.error(
                                    offset,
                                    format!(
                                        "{} does not accept generic argument named '{}'",
                                        target, name
                                    ),
                                );
                            }
                        }

                        // Make a parselet instance from the instance definition;
                        // This can be the final parselet instance, but constants
                        // might contain generic references as well, which are being
                        // resolved during further compilation and derivation.
                        let parselet = ImlValue::from(ImlParselet::new(ImlParseletInstance {
                            model: parselet.model.clone(),
                            generics,
                            offset: parselet.offset.clone(),
                            name: parselet.name.clone(),
                            severity: severity.unwrap_or(parselet.severity),
                            is_generated: *is_generated,
                        }));

                        Some(parselet)
                    }
                    Some(target) => {
                        scope.error(*offset, format!("Cannot create instance from '{}'", target));
                        None
                    }
                    None => None,
                }
            }
            _ => Some(self.clone()),
        }
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
            Self::Shared(value) => value.borrow().is_callable(without_arguments),
            Self::SelfValue | Self::SelfToken => true, // fixme?
            Self::Value(value) => value.is_callable(without_arguments),
            Self::Parselet(parselet) => {
                let parselet = parselet.borrow();
                let parselet = parselet.model.borrow();

                if without_arguments {
                    parselet.signature.len() == 0
                        || parselet.signature.iter().all(|arg| arg.1.is_some())
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
            Self::Shared(value) => value.borrow().is_consuming(),
            Self::SelfValue => false,
            Self::SelfToken | Self::VoidToken => true,
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
            ImlValue::Shared(value) => {
                return value.borrow().compile(program, current, offset, call, ops)
            }
            ImlValue::VoidToken => ops.push(Op::Next),
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
                    .as_ref()
                    .unwrap()
                    .compile(program, current, offset, call, ops)
            }
            ImlValue::SelfValue | ImlValue::SelfToken | ImlValue::Parselet(_) => {}
            _ => unreachable!("{}", self),
        }

        // Check if something has been pushed before.
        if start == ops.len() {
            let idx = match self {
                ImlValue::SelfValue | ImlValue::SelfToken => current.1, // use current index
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
            Self::Shared(value) => value.borrow().fmt(f),
            Self::SelfValue => write!(f, "self"),
            Self::SelfToken => write!(f, "Self"),
            Self::VoidToken => write!(f, "Void"),
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
            Self::Shared(value) => value.borrow().hash(state),
            Self::VoidToken => state.write_u8('V' as u8),
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
            other => unreachable!("{:?} is unhashable", other),
        }
    }
}

impl From<RefValue> for ImlValue {
    fn from(value: RefValue) -> Self {
        Self::Value(value)
    }
}
