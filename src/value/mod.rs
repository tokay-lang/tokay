//! Tokay value and object representation
use std::cell::RefCell;
use std::rc::Rc;

use crate::builtin::Builtin;
use crate::error::Error;
use crate::vm::{Accept, Context, Reject};

mod dict;
mod list;
mod method;
mod parselet;
mod string;
mod token;

pub use dict::Dict;
pub use list::List;
pub use method::Method;
pub use parselet::Parselet;
pub use string::String;
pub use token::Token;

// Object
// ----------------------------------------------------------------------------

/// Describes an interface to a callable object.
pub trait Object {
    fn name(&self) -> &str;

    fn repr(&self) -> String {
        format!("\"<{} {:p}>\"", self.name(), self)
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
    Integer(i64), // int
    Float(f64),   // float
    Addr(usize),  // addr

    // Objects
    String(String),  // str
    List(Box<List>), // list
    Dict(Box<Dict>), // dict

    // Callables
    Token(Box<Token>),               // Token
    Parselet(Rc<RefCell<Parselet>>), // Parselet
    Builtin(&'static Builtin),       // Builtin
    Method(Box<Method>),             // Method
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
    // Retieve type name of a value
    pub fn name(&self) -> &str {
        match self {
            Self::Void => "void",
            Self::Null => "null",
            Self::True => "true",
            Self::False => "false",
            Self::Integer(_) => "int",
            Self::Float(_) => "float",
            Self::Addr(_) => "addr",
            // fixme: vv-- below is bullshit area, as this could be trait object? --vv //
            Self::String(s) => s.name(),
            Self::List(l) => l.name(),
            Self::Dict(d) => d.name(),
            Self::Token(t) => t.name(),
            Self::Parselet(_) => "parselet",
            Self::Builtin(_) => "builtin",
            Self::Method(m) => m.name(),
        }
    }

    /// Get representation in Tokay code.
    pub fn repr(&self) -> String {
        match self {
            Self::Void => "void".to_string(),
            Self::Null => "null".to_string(),
            Self::True => "true".to_string(),
            Self::False => "false".to_string(),
            Self::Integer(i) => format!("{}", i),
            Self::Addr(a) => format!("{}", a),
            Self::Float(f) => format!("{}", f),
            // fixme: vv-- below is bullshit area, as this could be trait object? --vv //
            Self::String(s) => s.repr(),
            Self::List(l) => l.repr(),
            Self::Dict(d) => d.repr(),
            Self::Token(t) => t.repr(),
            Self::Parselet(p) => p.borrow().repr(),
            Self::Builtin(b) => format!("<builtin {}>", b.name),
            Self::Method(m) => m.repr(),
        }
    }

    /// Get a value's boolean meaning.
    pub fn is_true(&self) -> bool {
        match self {
            Self::True => true,
            Self::Integer(i) => *i != 0,
            Self::Float(f) => *f != 0.0,
            // fixme: vv-- below is bullshit area, as this could be trait object? --vv //
            Self::String(s) => s.len() > 0,
            Self::List(l) => l.len() > 0,
            Self::Dict(d) => d.len() > 0,
            Self::Builtin(_) | Self::Parselet(_) | Self::Addr(_) | Self::Method(_) => true,
            _ => false,
        }
    }

    /// Get value's integer representation.
    pub fn to_i64(&self) -> i64 {
        match self {
            Self::True => 1,
            Self::Integer(i) => *i,
            Self::Float(f) => *f as i64,
            Self::String(s) => {
                // todo: JavaScript-style parseInt-like behavior?
                match s.parse::<i64>() {
                    Ok(i) => i,
                    Err(_) => 0,
                }
            }
            _ => 0,
        }
    }

    /// Get value's float representation.
    pub fn to_f64(&self) -> f64 {
        match self {
            Self::True => 1.0,
            Self::Integer(i) => *i as f64,
            Self::Float(f) => *f,
            Self::String(s) => {
                // todo: JavaScript-style parseFloat-like behavior?
                match s.parse::<f64>() {
                    Ok(f) => f,
                    Err(_) => 0.0,
                }
            }
            _ => 0.0,
        }
    }

    /// Get value's usize representation.
    pub fn to_usize(&self) -> usize {
        match self {
            Self::True => 1,
            Self::Integer(i) => *i as usize,
            Self::Float(f) => *f as usize,
            Self::Addr(a) => *a,
            Self::String(s) => {
                // todo: JavaScript-style parseInt-like behavior?
                match s.parse::<usize>() {
                    Ok(i) => i,
                    Err(_) => 0,
                }
            }
            _ => 0,
        }
    }

    pub fn to_string(&self) -> String {
        if let Self::String(s) = self {
            s.clone()
        } else {
            self.repr()
        }
    }

    /// Retrieve &str from a value in case it is a string.
    pub fn str(&self) -> Option<&str> {
        if let Self::String(s) = self {
            Some(&s)
        } else {
            None
        }
    }

    /// Retrieve &List from a value in case it is a list.
    pub fn list(&self) -> Option<&List> {
        if let Self::List(l) = self {
            Some(&l)
        } else {
            None
        }
    }

    /// Retrieve &Dict from a value in case it is a dict.
    pub fn dict(&self) -> Option<&Dict> {
        if let Self::Dict(d) = self {
            Some(&d)
        } else {
            None
        }
    }

    /*
    /// Retrieve a method from a value.
    pub fn get_method(&self, method: &str) -> Result<Value, String> {
        let name = format!("{}_{}", self.name(), method);

        if let Some(builtin) = builtin::get(&name) {
            return Ok(Value::Method(Box::new(Method {
                object: self.clone(),
                method: Value::Builtin(builtin).into(),
            })));
        }

        Err(format!("Method '{}' not found", name))
    }
    */

    /// Check whether a value is callable, and when its callable if with or without arguments.
    pub fn is_callable(&self, with_arguments: bool) -> bool {
        match self {
            Value::Token(tok) => tok.is_callable(with_arguments),
            Value::Builtin(_) => true,
            Value::Parselet(parselet) => parselet.borrow().is_callable(with_arguments),
            Value::Method(method) => method.is_callable(with_arguments),
            _ => false,
        }
    }

    /// Check whether a value is consuming
    pub fn is_consuming(&self) -> bool {
        match self {
            Value::Token(tok) => tok.is_consuming(),
            Value::Builtin(builtin) => builtin.is_consumable(),
            Value::Parselet(parselet) => parselet.borrow().is_consuming(),
            Value::Method(method) => method.is_consuming(),
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
            Value::Token(token) => token.call(context, args, nargs),
            Value::Builtin(builtin) => builtin.call(context, args, nargs),
            Value::Parselet(parselet) => parselet.borrow().call(context, args, nargs),
            Value::Method(method) => method.call(context, args, nargs),
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
            (Value::Float(a), b) => Ok(Value::Float(a + b.to_f64())),
            (a, Value::Float(b)) => Ok(Value::Float(a.to_f64() + b)),

            // All is threatened as Integer
            (a, b) => Ok(Value::Integer(a.to_i64() + b.to_i64())),
        }
    }

    // Substraction
    pub fn sub(&self, rhs: &Value) -> Result<Value, Error> {
        match (self, rhs) {
            // When one is Float...
            (Value::Float(a), b) => Ok(Value::Float(a - b.to_f64())),
            (a, Value::Float(b)) => Ok(Value::Float(a.to_f64() - b)),

            // All is threatened as Integer
            (a, b) => Ok(Value::Integer(a.to_i64() - b.to_i64())),
        }
    }

    // Multiplication
    pub fn mul(&self, rhs: &Value) -> Result<Value, Error> {
        match (self, rhs) {
            // When one is String and one is something else...
            (Value::String(s), n) | (n, Value::String(s)) => {
                Ok(Value::String(s.repeat(n.to_usize())))
            }

            // When one is Float...
            (Value::Float(a), b) => Ok(Value::Float(a * b.to_f64())),
            (a, Value::Float(b)) => Ok(Value::Float(a.to_f64() * b)),

            // All is threatened as Integer
            (a, b) => Ok(Value::Integer(a.to_i64() * b.to_i64())),
        }
    }

    // Division
    pub fn div(&self, rhs: &Value) -> Result<Value, Error> {
        match (self, rhs) {
            // When one is Float...
            (Value::Float(_), _) | (_, Value::Float(_)) => {
                let a = self.to_f64();
                let b = rhs.to_f64();

                if b == 0.0 {
                    return Err(Error::new(None, "Cannot divide by zero".to_string()));
                }

                Ok(Value::Float(a / b))
            }

            // ...otherwise, all is assumed as integer.
            (a, b) => {
                let a = a.to_i64();
                let b = b.to_i64();

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
            v => Ok(Value::Integer(-v.to_i64())),
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

/// Convert a RefValue into a Value
impl From<RefValue> for Value {
    fn from(value: RefValue) -> Self {
        match Rc::try_unwrap(value) {
            Ok(value) => value.into_inner(),
            Err(value) => value.borrow().clone(),
        }
    }
}

// Conversion from native types into Value

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

impl From<List> for Value {
    fn from(value: List) -> Self {
        Value::List(Box::new(value))
    }
}

impl From<Dict> for Value {
    fn from(value: Dict) -> Self {
        Value::Dict(Box::new(value))
    }
}

// This is bat country...
/*
#[test]
fn test_literal() {
    assert_eq!(Value::Integer(1337).to_i64().unwrap(), Some(1337));
    assert_eq!(Value::Integer(-1337).to_i64().unwrap(), Some(-1337));
    //assert_eq!(Value::Float(13.37).to_f64(), 13.37);
    //assert_eq!(Value::Float(-13.37).to_f64(), -13.37);
    assert_eq!(Value::String("hello world".to_string()).to_string().unwrap(), Some("hello world".to_string()));
    assert_eq!(Value::Void.to_string().unwrap(), Some("void".to_string()));
    assert_eq!(Value::True.to_string().unwrap(), Some("true".to_string()));
    assert_eq!(Value::False.to_string().unwrap(), Some("false".to_string()));
}

#[test]
fn test_conversion() {
    let v = Value::Void;
    assert_eq!(v.to_string(), Some("void".to_string()));
    assert_eq!(v.to_i64(), None);
    assert_eq!(v.to_bool(), Some(false));

    /*
    let f = Value::Float(1.337);
    assert_eq!(f.to_f64(), 1.337);
    assert_eq!(f.to_string(), "1.337");
    assert_eq!(f.to_f64(), 1.337);
    assert_eq!(f.to_i64(), 1);
    */

    let s = Value::String("42".to_string());
    assert_eq!(s.to_string(), Some("42".to_string()));
    assert_eq!(s.to_i64(), Some(42));
    assert_eq!(s.to_f64(), Some(42.0));
    assert_eq!(s.to_string(), Some("42".to_string()));

    let s = Value::String("gunshot42".to_string());
    assert_eq!(s.to_string(), Some("gunshot42".to_string()));
    assert_eq!(s.to_i64(), Some(0));
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
