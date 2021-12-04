//! Tokay value and object representation
mod parselet;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use crate::builtin::{self, Builtin};
use crate::error::Error;
use crate::token::Token;
use crate::vm::{Accept, Capture, Context, Reject};
pub use parselet::Parselet;

pub type RefValue = Rc<RefCell<Value>>;
pub type List = Vec<RefValue>;
pub type Dict = BTreeMap<String, RefValue>;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    // Atomics
    Void,  // void
    Null,  // null
    True,  // true
    False, // false

    // Primitives
    Integer(i64), // integer
    Float(f64),   // float
    Addr(usize),  // usize

    // Objects
    String(String),  // string
    List(Box<List>), // list
    Dict(Box<Dict>), // dict

    // Callables
    Token(Box<Token>),                 // Token
    Parselet(Rc<RefCell<Parselet>>),   // Parselet
    Builtin(&'static Builtin),         // Builtin
    Method(Box<(RefValue, RefValue)>), // Method
}

/** Value construction helper-macro

This macro is used to easily construct Tokay values in Rust code.

Examples:
```
use tokay::value::*;
use tokay::value;

let i = value![1];
let s = value!("String");
let l = value![[1, 2, 3]];
let d = value![["a" => 1, "b" => 2, "c" => 3]];
```
*/
#[macro_export]
macro_rules! value {
    ( [ $($key:literal => $value:tt),* ] ) => {
        {
            let mut dict = Dict::new();
            $( dict.insert($key.to_string(), value!($value).into_refvalue()); )*
            Value::Dict(Box::new(dict))
        }
    };

    ( [ $($value:tt),* ] ) => {
        {
            let mut list = List::new();
            $( list.push(value!($value).into_refvalue()); )*
            Value::List(Box::new(list))
        }
    };

    ( $value:expr ) => {
        if let Some(value) = (&$value as &dyn std::any::Any).downcast_ref::<bool>()
        {
            if *value {
                Value::True
            }
            else {
                Value::False
            }
        }
        else
        if let Some(value) = (&$value as &dyn std::any::Any).downcast_ref::<f32>()
        {
            Value::Float(*value as f64)
        }
        else
        if let Some(value) = (&$value as &dyn std::any::Any).downcast_ref::<f64>()
        {
            Value::Float(*value)
        }
        else
        if let Some(value) = (&$value as &dyn std::any::Any).downcast_ref::<i32>()
        {
            Value::Integer(*value as i64)
        }
        else
        if let Some(value) = (&$value as &dyn std::any::Any).downcast_ref::<i64>()
        {
            Value::Integer(*value)
        }
        else
        if let Some(value) = (&$value as &dyn std::any::Any).downcast_ref::<usize>()
        {
            Value::Addr(*value)
        }
        else {
            Value::String($value.to_string())
        }
    }
}

/*
impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Void => write!(f, "void"),
            Value::Null => write!(f, "null"),
            Value::True => write!(f, "true"),
            Value::False => write!(f, "false"),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(v) => write!(f, "{}", v),
            Value::Addr(a) => write!(f, "{}", a),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::List(l) => {
                write!(f, "(")?;

                for (i, v) in l.iter().enumerate() {
                    write!(f, "{:?}", v)?;

                    if i + 1 < l.len() {
                        write!(f, ", ")?;
                    }
                }

                write!(f, ")")
            },
            Value::Dict(d) => {
                write!(f, "(")?;

                for (i, (k, v)) in d.iter().enumerate() {
                    write!(f, "{:?}: {:?}", k, v)?;

                    if i + 1 < d.len() {
                        write!(f, ", ")?;
                    }
                }

                write!(f, ")")
            },
            Value::Parselet(p) => write!(f, "@{:?}", p)
        }
    }
}
*/

impl Value {
    /// Create a RefValue from a Value.
    pub fn into_refvalue(self) -> RefValue {
        RefValue::new(RefCell::new(self))
    }

    /// Convert a RefValue into a Value
    pub fn from_ref(this: RefValue) -> Result<Value, RefValue> {
        match Rc::try_unwrap(this) {
            Ok(this) => Ok(this.into_inner()),
            Err(this) => Err(this),
        }
    }

    // Create a Value from a bool
    pub fn from_bool(b: bool) -> Value {
        if b {
            Value::True
        } else {
            Value::False
        }
    }

    /** Create an Ok(Accept::Push) from a value. This function is a shortcut. **/
    pub fn into_accept_push_capture(self) -> Result<Accept, Reject> {
        Ok(Accept::Push(Capture::Value(self.into_refvalue(), None, 10)))
    }

    /// Check if value is void.
    pub fn is_void(&self) -> bool {
        matches!(self, Value::Void)
    }

    /// Get a value's boolean meaning.
    pub fn is_true(&self) -> bool {
        match self {
            Self::True => true,
            Self::Integer(i) => *i != 0,
            Self::Float(f) => *f != 0.0,
            Self::String(s) => s.len() > 0,
            Self::List(l) => l.len() > 0,
            Self::Dict(d) => d.len() > 0,
            Self::Builtin(_) | Self::Parselet(_) | Self::Addr(_) => true,
            _ => false,
        }
    }

    /// Get value's integer representation.
    pub fn to_integer(&self) -> i64 {
        match self {
            Self::True => 1,
            Self::Integer(i) => *i,
            Self::Float(f) => *f as i64,
            Self::String(s) => {
                // todo: parseInt-like behavior?
                match s.parse::<i64>() {
                    Ok(i) => i,
                    Err(_) => 0,
                }
            }
            _ => 0,
        }
    }

    /// Get value's float representation.
    pub fn to_float(&self) -> f64 {
        match self {
            Self::True => 1.0,
            Self::Integer(i) => *i as f64,
            Self::Float(f) => *f,
            Self::String(s) => {
                // todo: parseFloat-like behavior?
                match s.parse::<f64>() {
                    Ok(f) => f,
                    Err(_) => 0.0,
                }
            }
            _ => 0.0,
        }
    }

    /// Get value's usize representation.
    pub fn to_addr(&self) -> usize {
        match self {
            Self::True => 1,
            Self::Integer(i) => *i as usize,
            Self::Float(f) => *f as usize,
            Self::Addr(a) => *a,
            Self::String(s) => {
                // todo: parseInt-like behavior?
                match s.parse::<usize>() {
                    Ok(i) => i,
                    Err(_) => 0,
                }
            }
            _ => 0,
        }
    }

    // Get value's representation as Tokay code.
    pub fn repr(&self) -> String {
        match self {
            Self::Void => "void".to_string(),
            Self::String(s) => {
                let mut ret = String::with_capacity(s.len() + 2);
                ret.push('"');

                for ch in s.chars() {
                    match ch {
                        '\"' => ret.push_str("!!"),
                        '\n' => ret.push_str("\\n"),
                        '\r' => ret.push_str("\\r"),
                        '\t' => ret.push_str("\\t"),
                        ch => ret.push(ch),
                    }
                }

                ret.push('"');
                ret
            }
            Self::List(l) => {
                let mut ret = "(".to_string();
                for item in l.iter() {
                    if ret.len() > 1 {
                        ret.push_str(", ");
                    }

                    ret.push_str(&item.borrow().repr());
                }

                if l.len() == 1 {
                    ret.push_str(", ");
                }

                ret.push(')');
                ret
            }
            Self::Dict(d) => {
                let mut ret = "(".to_string();
                for (key, value) in d.iter() {
                    if ret.len() > 1 {
                        ret.push_str(", ");
                    }

                    // todo: Put this into a utility function...
                    if !key.chars().all(|ch| ch.is_alphabetic()) {
                        ret.push('"');
                        for ch in key.chars() {
                            match ch {
                                '\n' => ret.push_str("\\n"),
                                '\r' => ret.push_str("\\r"),
                                '\t' => ret.push_str("\\t"),
                                '"' => ret.push_str("\\\""),
                                _ => ret.push(ch),
                            }
                        }
                        ret.push('"');
                    } else {
                        ret.push_str(key);
                    }

                    ret.push_str(" => ");
                    ret.push_str(&value.borrow().repr());
                }
                ret.push(')');
                ret
            }
            Self::Parselet(p) => {
                let p = &*p.borrow();
                if let Some(name) = p.name.as_ref() {
                    format!("<parselet {}>", name)
                } else {
                    format!("<parselet {}>", p as *const Parselet as usize)
                }
            }
            Self::Builtin(b) => format!("<builtin {}>", b.name),
            Self::Method(m) => format!("<method {}.{}>", m.0.borrow().repr(), m.1.borrow().repr()),
            other => other.to_string(),
        }
    }

    /// Get value's string representation.
    pub fn to_string(&self) -> String {
        match self {
            Self::Void => "".to_string(),
            Self::Null => "null".to_string(),
            Self::True => "true".to_string(),
            Self::False => "false".to_string(),
            Self::Integer(i) => format!("{}", i),
            Self::Addr(a) => format!("{}", a),
            Self::Float(f) => format!("{}", f),
            Self::String(s) => s.clone(),
            other => other.repr(),
        }
    }

    /// Get a value's list representation.
    pub fn to_list(&self) -> List {
        if let Self::List(l) = self {
            *l.clone()
        } else {
            let mut l = List::new();
            l.push(self.clone().into_refvalue());
            l
        }
    }

    /// Get a value's dict representation.
    pub fn to_dict(&self) -> Dict {
        if let Self::Dict(d) = self {
            *d.clone()
        } else {
            let mut d = Dict::new();
            //fixme "0"?
            d.insert("0".to_string(), self.clone().into_refvalue());
            d
        }
    }

    /// Turn value into raw String, consuming the value.
    pub fn into_string(self) -> String {
        if let Self::String(s) = self {
            s
        } else {
            self.to_string()
        }
    }

    /// Turn value into raw List, consuming the value.
    pub fn into_list(self) -> List {
        if let Self::List(l) = self {
            *l
        } else {
            self.to_list()
        }
    }

    /// Turn value into raw Dict, consuming the value.
    pub fn into_dict(self) -> Dict {
        if let Self::Dict(d) = self {
            *d
        } else {
            self.to_dict()
        }
    }

    /// Extract String from a value
    pub fn get_string(&self) -> Option<&str> {
        if let Self::String(s) = self {
            Some(&s)
        } else {
            None
        }
    }

    /// Extract List from a value
    pub fn get_list(&self) -> Option<&List> {
        if let Self::List(l) = self {
            Some(&l)
        } else {
            None
        }
    }

    /// Extract Dict from a value
    pub fn get_dict(&self) -> Option<&Dict> {
        if let Self::Dict(d) = self {
            Some(&d)
        } else {
            None
        }
    }

    /// Retrieve index from a value.
    pub fn get_index(&self, index: &Value) -> Result<RefValue, String> {
        match self {
            Self::String(s) => {
                let index = index.to_addr();
                if let Some(ch) = s.chars().nth(index) {
                    Ok(Value::String(format!("{}", ch)).into_refvalue())
                } else {
                    Err(format!("Index {} beyond end of string", index))
                }
            }
            Self::List(l) => {
                let index = index.to_addr();
                if let Some(value) = l.get(index) {
                    Ok(value.clone())
                } else {
                    Err(format!("Index {} out of bounds", index))
                }
            }
            Self::Dict(d) => {
                let index = index.to_string();
                if let Some(value) = d.get(&index) {
                    Ok(value.clone())
                } else {
                    Err(format!("Key '{}' not found", index))
                }
            }
            _ => Err(format!("Value '{}' doesn't allow indexing", self.repr())),
        }
    }

    // Set value to an index.
    pub fn set_index(&mut self, index: &Value, value: RefValue) -> Result<(), String> {
        // fixme: Incomplete, concept missing.
        match self {
            Self::String(s) => {
                let index = index.to_addr();
                if index < s.len() {
                    todo!();
                    Ok(())
                } else {
                    Err(format!("Index {} beyond end of string", index))
                }
            }
            Self::List(l) => {
                let index = index.to_addr();
                if index < l.len() {
                    l[index] = value;
                    Ok(())
                } else if index == l.len() {
                    l.push(value);
                    Ok(())
                } else {
                    Err(format!("Index {} out of bounds", index))
                }
            }
            Self::Dict(d) => {
                let index = index.to_string();
                d.insert(index, value);
                Ok(())
            }
            _ => Err(format!("Value '{}' doesn't allow indexing", self.repr())),
        }
    }

    /// Retrieve attribute from a refvalue.
    /// Currently this is only a built-in mapping with a value.
    pub fn get_attr(this: RefValue, attr: &Value) -> Result<RefValue, String> {
        let value = &*this.borrow();

        let prefix = match value {
            Self::String(_) => "str",
            Self::Dict(_) => "dict",
            Self::List(_) => "list",
            _ => {
                return Err(format!(
                    "Value '{}' doesn't allow for attribute access",
                    value.repr()
                ))
            }
        };

        let attr = attr.get_string().unwrap();
        match attr {
            "len" => Ok(Value::Addr(value.get_attr_len()).into_refvalue()),
            _ => {
                let name = format!("{}_{}", prefix, attr);

                if let Some(builtin) = builtin::get(&name) {
                    return Ok(Value::Method(Box::new((
                        this.clone(),
                        Value::Builtin(builtin).into_refvalue(),
                    )))
                    .into_refvalue());
                }

                Err(format!("Method '{}' not found", name))
            }
        }
    }

    pub fn get_attr_len(&self) -> usize {
        match self {
            Value::String(s) => s.chars().count(),
            Value::Dict(d) => d.len(),
            Value::List(l) => l.len(),
            _ => 0,
        }
    }

    /// Check whether a value is callable, and when its callable if with or without arguments.
    pub fn is_callable(&self, with_arguments: bool) -> bool {
        match self {
            Value::Token(_) => !with_arguments,
            Value::Builtin(_) => true,
            Value::Parselet(parselet) => parselet.borrow().is_callable(with_arguments),
            Value::Method(method) => method.1.borrow().is_callable(with_arguments),
            _ => false,
        }
    }

    /// Check whether a value is consuming
    pub fn is_consuming(&self) -> bool {
        match self {
            Value::Token(_) => true,
            Value::Builtin(builtin) => builtin.is_consumable(),
            Value::Parselet(parselet) => parselet.borrow().consuming.is_some(),
            _ => false,
        }
    }

    /// Call a value with a given context, argument and named argument set.
    pub fn call(
        &self,
        context: &mut Context,
        args: usize,
        nargs: Option<Dict>,
    ) -> Result<Accept, Reject> {
        match self {
            Value::Token(token) => {
                assert!(args == 0 && nargs.is_none());
                token.read(context.runtime.reader)
            }
            Value::Builtin(builtin) => builtin.call(context, args, nargs),
            Value::Parselet(parselet) => {
                parselet
                    .borrow()
                    .run(context.runtime, args, nargs, false, context.depth + 1)
            }
            Value::Method(method) => {
                // Method call injects the related object into the stack and calls the method afterwards.
                context.runtime.stack.insert(
                    context.runtime.stack.len() - args,
                    Capture::Value(method.0.clone(), None, 0),
                );
                method.1.borrow().call(context, args + 1, nargs)
            }
            _ => Error::new(None, format!("'{}' cannot be called", self.repr())).into_reject(),
        }
    }

    // The following functions where prevously solved by std::ops::*, etc. but this
    // is now changed as error handling must work with Tokay's VM.
    // Fixme: For better integration with Rust, std::ops::* could be re-implemented
    // by wrapping these operational functions.

    // Addition
    pub fn add(&self, rhs: &Value) -> Result<Value, Error> {
        match (self, rhs) {
            // When one is String...
            (Value::String(a), b) => Ok(Value::String(a.to_owned() + &b.to_string())),
            (a, Value::String(b)) => Ok(Value::String(a.to_string() + &b)),

            // When one is Float...
            (Value::Float(a), b) => Ok(Value::Float(a + b.to_float())),
            (a, Value::Float(b)) => Ok(Value::Float(a.to_float() + b)),

            // All is threatened as Integer
            (a, b) => Ok(Value::Integer(a.to_integer() + b.to_integer())),
        }
    }

    // Substraction
    pub fn sub(&self, rhs: &Value) -> Result<Value, Error> {
        match (self, rhs) {
            // When one is Float...
            (Value::Float(a), b) => Ok(Value::Float(a - b.to_float())),
            (a, Value::Float(b)) => Ok(Value::Float(a.to_float() - b)),

            // All is threatened as Integer
            (a, b) => Ok(Value::Integer(a.to_integer() - b.to_integer())),
        }
    }

    // Multiplication
    pub fn mul(&self, rhs: &Value) -> Result<Value, Error> {
        match (self, rhs) {
            // When one is String and one is something else...
            (Value::String(s), n) | (n, Value::String(s)) => {
                Ok(Value::String(s.repeat(n.to_addr())))
            }

            // When one is Float...
            (Value::Float(a), b) => Ok(Value::Float(a * b.to_float())),
            (a, Value::Float(b)) => Ok(Value::Float(a.to_float() * b)),

            // All is threatened as Integer
            (a, b) => Ok(Value::Integer(a.to_integer() * b.to_integer())),
        }
    }

    // Division
    pub fn div(&self, rhs: &Value) -> Result<Value, Error> {
        match (self, rhs) {
            // When one is Float...
            (Value::Float(_), _) | (_, Value::Float(_)) => {
                let a = self.to_float();
                let b = rhs.to_float();

                if b == 0.0 {
                    return Err(Error::new(None, "Cannot divide by zero".to_string()));
                }

                Ok(Value::Float(a / b))
            }

            // All is threatened as Integer
            (a, b) => {
                let a = a.to_integer();
                let b = b.to_integer();

                if b == 0 {
                    return Err(Error::new(None, "Cannot divide by zero".to_string()));
                }

                // If there's no remainder, perform an integer division
                if a % b == 0 {
                    Ok(Value::Integer(a / b))
                }
                // Otherwise a floating point division
                else {
                    Ok(Value::Float(a as f64 / b as f64))
                }
            }
        }
    }

    // Negation
    pub fn neg(&self) -> Result<Value, Error> {
        match self {
            Value::Float(v) => Ok(Value::Float(-v)),
            v => Ok(Value::Integer(-v.to_integer())),
        }
    }

    // Logical not
    pub fn not(&self) -> Result<Value, Error> {
        match self {
            //Value::Integer(i) => Value::Integer(!i),  // breaks semantics
            //Value::Addr(a) => Value::Addr(!a),
            v => Ok(if v.is_true() {
                Value::False
            } else {
                Value::True
            }),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

// This is bat country...
/*
#[test]
fn test_literal() {
    assert_eq!(Value::Integer(1337).to_integer().unwrap(), Some(1337));
    assert_eq!(Value::Integer(-1337).to_integer().unwrap(), Some(-1337));
    //assert_eq!(Value::Float(13.37).to_float(), 13.37);
    //assert_eq!(Value::Float(-13.37).to_float(), -13.37);
    assert_eq!(Value::String("hello world".to_string()).to_string().unwrap(), Some("hello world".to_string()));
    assert_eq!(Value::Void.to_string().unwrap(), Some("void".to_string()));
    assert_eq!(Value::True.to_string().unwrap(), Some("true".to_string()));
    assert_eq!(Value::False.to_string().unwrap(), Some("false".to_string()));
}

#[test]
fn test_conversion() {
    let v = Value::Void;
    assert_eq!(v.to_string(), Some("void".to_string()));
    assert_eq!(v.to_integer(), None);
    assert_eq!(v.to_bool(), Some(false));

    /*
    let f = Value::Float(1.337);
    assert_eq!(f.to_float(), 1.337);
    assert_eq!(f.to_string(), "1.337");
    assert_eq!(f.to_float(), 1.337);
    assert_eq!(f.to_integer(), 1);
    */

    let s = Value::String("42".to_string());
    assert_eq!(s.to_string(), Some("42".to_string()));
    assert_eq!(s.to_integer(), Some(42));
    assert_eq!(s.to_float(), Some(42.0));
    assert_eq!(s.to_string(), Some("42".to_string()));

    let s = Value::String("gunshot42".to_string());
    assert_eq!(s.to_string(), Some("gunshot42".to_string()));
    assert_eq!(s.to_integer(), Some(0));
}

#[test]
fn test_add() {
    let x = Value::Integer(42);
    let y = Value::String("hello".to_string());
    assert_eq!(x + y, Value::String("42hello".to_string()));

    let x = Value::Integer(42);
    let y = Value::String("hello".to_string());
    assert_eq!(y + x, Value::String("hello42".to_string()));

    let x = Value::Integer(42);
    let y = Value::Integer(23);
    assert_eq!(x + y, Value::Integer(65));

    //let x = Value::Integer(42);
    //let y = Value::Float(1.337);
    //assert_eq!(x + y, Value::Float(43.337));
}


#[test]
fn test_mul() {
    let x = Value::Integer(42);
    let y = Value::Integer(23);
    assert_eq!(x * y, Value::Integer(966));

    let x = Value::Integer(3);
    let y = Value::String("hello".to_string());
    assert_eq!(x * y, Value::String("hellohellohello".to_string()));

    assert_eq!(Value::Bool(true) * Value::String("hello".to_string()), Value::String("hello".to_string()));
    assert_eq!(Value::Bool(false) * Value::String("hello".to_string()), Value::String("".to_string()));

    //let x = Value::Integer(42);
    //let y = Value::Float(1.337);
    //assert_eq!(x + y, Value::Float(43.337));
}

*/

/*
#[derive(Clone, PartialEq)]
pub struct RefValue(Rc<RefCell<Value>>);

impl RefValue {
    pub fn new(value: Value) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub fn borrow(&self) -> Ref<Value> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<Value> {
        self.0.borrow_mut()
    }

    pub fn into_value(this: RefValue) -> Result<Value, RefValue> {
        match Rc::try_unwrap(this.0) {
            Ok(this) => Ok(this.into_inner()),
            Err(this) => Err(RefValue(this))
        }
    }
}

impl std::fmt::Debug for RefValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.borrow())
    }
}
*/
