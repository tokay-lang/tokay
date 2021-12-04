//! Tokay value and object representation
use std::cell::RefCell;
use std::rc::Rc;

use crate::builtin::{self, Builtin};
use crate::error::Error;
use crate::vm::{Accept, Capture, Context, Reject};

mod dict;
mod list;
mod parselet;
mod string;
mod token;

pub use dict::Dict;
pub use list::List;
pub use parselet::Parselet;
pub use string::String;
pub use token::Token;

// Object
// ----------------------------------------------------------------------------

/// Describes an interface to communicate with an object.

pub trait Object {
    /// Return a name for the object
    fn name(&self) -> &str;

    /// Render a Tokay code representation for the object
    fn repr(&self) -> String {
        self.name().to_string()
    }

    /// Check whether the object represents true or false
    fn is_true(&self) -> bool {
        true
    }

    /// Return reference to String when object _is_ a string
    fn is_string(&self) -> Option<&String> {
        None
    }

    /// Return reference to List when object _is_ a List
    fn is_list(&self) -> Option<&List> {
        None
    }

    /// Return reference to Dict when object _is_ a Dict
    fn is_dict(&self) -> Option<&Dict> {
        None
    }

    /// Get object's integer value
    fn to_integer(&self) -> i64 {
        0
    }

    /// Get object's addr value
    fn to_addr(&self) -> usize {
        0
    }

    /// Get object's float value
    fn to_float(&self) -> f64 {
        0.0
    }

    /// Get object's string value
    fn to_string(&self) -> String {
        self.repr()
    }

    /// Return reference to List when object is a List
    fn to_list(&self) -> List {
        List::new()
    }

    /// Return reference to Dict when object is a Dict
    fn to_dict(&self) -> Dict {
        Dict::new()
    }

    /// Get an index from an object.
    fn get_index(&self, _index: &Value) -> Result<RefValue, String> {
        Err(format!("'{}' doesn't allow indexing", self.repr()))
    }

    /// Set a value to an index of an object.
    fn set_index(&mut self, _index: &Value, _value: RefValue) -> Result<(), String> {
        Err(format!("'{}' doesn't allow indexing", self.repr()))
    }

    /// Get an attribute from an object.
    fn get_attr(&self, _index: &Value) -> Result<RefValue, String> {
        Err(format!("'{}' has no attributes", self.repr()))
    }

    /// Check whether a value is callable, and when its callable if with or without arguments.
    fn is_callable(&self, _with_arguments: bool) -> bool {
        false
    }

    /// Check whether a value is consuming
    fn is_consuming(&self) -> bool {
        false
    }

    /// Call a value with a given context, argument and named argument set.
    fn call(
        &self,
        _context: &mut Context,
        _args: usize,
        _nargs: Option<Dict>,
    ) -> Result<Accept, Reject> {
        Error::new(None, format!("'{}' cannot be called", self.repr())).into_reject()
    }

    /* fixme: required when below section is used.
    /// Create a boxed clone of the object
    fn clone_dyn(&self) -> Box<dyn Object>;
    */
}

/*
Value could make use of Box<dyn Object> as a trait object, but this requires implementation
of several other trait on Box<dyn Object>. But this looses the possibility of doing PartialEq
and PartialOrd on the current implementation, which IS important.

Here is the link for a playground started on this:
https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=4d7fda9b8391506736837f93124a16f4

fixme: Need help with this!

/*
impl Clone for Box<dyn Object> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

impl PartialEq for Box<dyn Object> {
    fn eq(&self, other: &Self) -> bool {
        //self.len() == other.len()
        todo!();
    }
}


impl PartialOrd for Box<dyn Object> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        //self.len().partial_cmp(&other.len())
        todo!();
    }
}

// https://github.com/rust-lang/rust/issues/31740#issuecomment-700950186
impl PartialEq<&Self> for Box<dyn Object> {
    fn eq(&self, other: &&Self) -> bool {
        //self.len() == other.len()
        todo!();
    }
}
*/
*/

// RefValue
// ----------------------------------------------------------------------------

/// This is the reference-counted, dynamically borrowable value used in most places within the Tokay VM.
pub type RefValue = Rc<RefCell<Value>>;

impl From<Value> for RefValue {
    fn from(value: Value) -> Self {
        Self::new(RefCell::new(value))
    }
}

// Value
// ----------------------------------------------------------------------------

/// Represents a Tokay primitive, which can also be an object with further specialization.
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
            $( dict.insert($key.to_string(), value!($value).into()); )*
            Value::Dict(Box::new(dict))
        }
    };

    ( [ $($value:tt),* ] ) => {
        {
            let mut list = List::new();
            $( list.push(value!($value).into()); )*
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

impl Value {
    /// Convert a RefValue into a Value
    pub fn from_ref(this: RefValue) -> Result<Value, RefValue> {
        match Rc::try_unwrap(this) {
            Ok(this) => Ok(this.into_inner()),
            Err(this) => Err(this),
        }
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
            Self::List(l) => l.is_true(),
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
            Self::String(s) => s.repr(),
            Self::List(l) => l.repr(),
            Self::Dict(d) => d.repr(),
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
            l.to_list()
        } else {
            let mut l = List::new();
            l.push(self.clone().into());
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
            d.insert("0".to_string(), self.clone().into());
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
            Self::String(s) => s.get_index(index),
            Self::List(l) => l.get_index(index),
            Self::Dict(d) => d.get_index(index),
            _ => Err(format!("Value '{}' doesn't allow indexing", self.repr())),
        }
    }

    // Set value to an index.
    pub fn set_index(&mut self, index: &Value, value: RefValue) -> Result<(), String> {
        // fixme: Incomplete, concept missing.
        match self {
            Self::String(s) => s.set_index(index, value),
            Self::List(l) => l.set_index(index, value),
            Self::Dict(d) => d.set_index(index, value),
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
            "len" => Ok(Value::Addr(value.get_attr_len()).into()),
            _ => {
                let name = format!("{}_{}", prefix, attr);

                if let Some(builtin) = builtin::get(&name) {
                    return Ok(Value::Method(Box::new((
                        this.clone(),
                        Value::Builtin(builtin).into(),
                    )))
                    .into());
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

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        if value {
            Value::True
        } else {
            Value::False
        }
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Value::Addr(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
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
