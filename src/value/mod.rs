//! Tokay value and object representation
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::builtin::Builtin;
use crate::error::Error;
use crate::vm::{Accept, Context, Reject};

mod dict;
mod list;
mod method;
mod object;
mod parselet;
mod string;
mod token;

pub use dict::Dict;
pub use list::List;
pub use method::Method;
pub use object::Object;
pub use parselet::{Parselet, ParseletRef};
pub use string::Str;
pub use token::Token;

// RefValue
// ----------------------------------------------------------------------------

/// This is the reference-counted, dynamically borrowable value used in most places within the Tokay VM.
#[derive(Clone, PartialEq, PartialOrd)]
pub struct RefValue {
    value: Rc<RefCell<Value>>,
}

impl RefValue {
    pub fn borrow(&self) -> Ref<Value> {
        self.value.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<Value> {
        self.value.borrow_mut()
    }

    /** Checks for a method on a value given by value type and method name.

    Methods are currently only native Rust functions provided via builtins.

    A method follows the naming convention <type>_<method>, so that the
    calls `"hello".upper()` and `str_upper("hello")` are calls to the
    same function.
    */
    pub fn get_method(&self, name: &str) -> Result<&'static Builtin, String> {
        let name = format!("{}_{}", self.value.borrow().name(), name);

        if let Some(builtin) = Builtin::get(&name) {
            Ok(builtin)
        } else {
            Err(format!("Method '{}' not found", name))
        }
    }

    /** Creates a callable Method object from a value and a given builtin. */
    pub fn create_method(&self, name: &str) -> Result<RefValue, String> {
        let builtin = self.get_method(name)?;
        return Ok(RefValue::from(Method {
            object: self.clone(),
            method: RefValue::from(builtin),
        }));
    }

    /** Performs a direct method call on a value.

    This function is designed to invoke methods on values directly from Rust code. */
    pub fn call_method(
        &self,
        name: &str,
        mut args: Vec<RefValue>,
    ) -> Result<Option<RefValue>, String> {
        let builtin = self.get_method(name)?;

        // Inject own value as first parameter.
        args.insert(0, self.clone());

        // Call the builtin directly.
        builtin.call(None, args)
    }

    /// Checks against a given type name
    pub fn is(&self, name: &str) -> bool {
        self.name() == name
    }

    /// Object ID (unique memory address)
    pub fn id(&self) -> usize {
        self.borrow().id()
    }

    /// Object type name.
    pub fn name(&self) -> &'static str {
        self.borrow().name()
    }

    /// Get representation in Tokay code.
    pub fn repr(&self) -> String {
        self.borrow().repr()
    }

    /// Get a value's boolean meaning.
    pub fn is_void(&self) -> bool {
        matches!(&*self.borrow(), Value::Void)
    }

    /// Get a value's boolean meaning.
    pub fn is_true(&self) -> bool {
        self.borrow().is_true()
    }

    /// Get value's integer representation.
    pub fn to_i64(&self) -> i64 {
        self.borrow().to_i64()
    }

    /// Get value's float representation.
    pub fn to_f64(&self) -> f64 {
        self.borrow().to_f64()
    }

    /// Get value's usize representation.
    pub fn to_usize(&self) -> usize {
        self.borrow().to_usize()
    }

    // Get value's String representation
    pub fn to_string(&self) -> String {
        self.borrow().to_string()
    }

    /// Check whether a value is object, and when its object if with or without arguments.
    pub fn is_callable(&self, with_arguments: bool) -> bool {
        self.borrow().is_callable(with_arguments)
    }

    /// Check whether a value is consuming
    pub fn is_consuming(&self) -> bool {
        self.borrow().is_consuming()
    }

    /// Check whether a value is consuming
    pub fn is_nullable(&self) -> bool {
        self.borrow().is_nullable()
    }

    /// Call a value with a given context, argument and named argument set.
    pub fn call(
        &self,
        context: &mut Context,
        args: usize,
        nargs: Option<Dict>,
    ) -> Result<Accept, Reject> {
        if let Value::Object(object) = &*self.borrow() {
            object.call(context, args, nargs)
        } else {
            Error::new(None, format!("'{}' cannot be called", self.repr())).into()
        }
    }

    // The following functions where previously solved by std::ops::*, etc. but this
    // is now changed as error handling must work with Tokay's VM.
    // Fixme: For better integration with Rust, std::ops::* could be re-implemented
    // by wrapping these operational functions.

    // Equality
    /*
    pub fn eq(&self, rhs: &RefValue) -> Result<bool, Error> {
        Some(match (self, rhs) {
            (Self::Void, Self::Void) => true,
            (Self::Null, Self::Null) => true,
            (Self::True, Self::True) => true,
            (Self::False, Self::False) => true,

            (Self::Integer(_), _) | (_, Self::Integer(_)) => self.to_i64() == rhs.to_i64(),
            (Self::Float(_), _) | (_, Self::Float(_)) => self.to_f64() == rhs.to_f64(),

            (Self::String(a), Self::String(b)) => a == b,
            (Self::String(a), b) => a == &b.to_string(),
            (a, Self::String(b)) => &a.to_string() == b,

            (Self::List(a), Self::List(b)) => a == b,
            (Self::List(a), b) => a == List::from(b),
            (a, Self::List(b)) => List::from(a) == b,

            (Self::Dict(a), Self::Dict(b)) => a == b,
            (Self::Dict(a), b) => a == Dict::from(b),
            (a, Self::Dict(b)) => Dict::from(a) == b,

            (a, b) => a.to_usize() == b.to_usize()
        })
    }
    */

    // Addition
    pub fn add(&self, rhs: RefValue) -> Result<RefValue, Error> {
        // todo: This must be moved to trait Object...
        match (&*self.borrow(), &*rhs.borrow()) {
            // When one is String...
            (Value::Str(a), b) => Ok(RefValue::from(a.as_str().to_owned() + &b.to_string())),
            (a, Value::Str(b)) => Ok(RefValue::from(a.to_string() + &b.as_str())),

            // When one is Float...
            (Value::Float(a), b) => Ok(Value::Float(a + b.to_f64()).into()),
            (a, Value::Float(b)) => Ok(Value::Float(a.to_f64() + b).into()),

            // All is threatened as Integer
            (a, b) => Ok(Value::Integer(a.to_i64() + b.to_i64()).into()),
        }
    }

    // Substraction
    pub fn sub(&self, rhs: RefValue) -> Result<RefValue, Error> {
        // todo: This must be moved to trait Object...
        match (&*self.borrow(), &*rhs.borrow()) {
            // When one is Float...
            (Value::Float(a), b) => Ok(Value::Float(a - b.to_f64()).into()),
            (a, Value::Float(b)) => Ok(Value::Float(a.to_f64() - b).into()),

            // All is threatened as Integer
            (a, b) => Ok(Value::Integer(a.to_i64() - b.to_i64()).into()),
        }
    }

    // Multiplication
    pub fn mul(&self, rhs: RefValue) -> Result<RefValue, Error> {
        // todo: This must be moved to trait Object...
        match (&*self.borrow(), &*rhs.borrow()) {
            // When one is String and one is something else...
            (Value::Str(s), n) | (n, Value::Str(s)) => {
                Ok(RefValue::from(s.as_str().to_owned().repeat(n.to_usize())))
            }

            // When one is Float...
            (Value::Float(a), _) => Ok(Value::Float(a * rhs.to_f64()).into()),
            (_, Value::Float(b)) => Ok(Value::Float(self.to_f64() * b).into()),

            // All is threatened as Integer
            (a, b) => Ok(Value::Integer(a.to_i64() * b.to_i64()).into()),
        }
    }

    // Division
    pub fn div(&self, rhs: RefValue) -> Result<RefValue, Error> {
        // todo: This must be moved to trait Object...
        match (&*self.borrow(), &*rhs.borrow()) {
            // When one is Float...
            (Value::Float(_), _) | (_, Value::Float(_)) => {
                let a = self.to_f64();
                let b = rhs.to_f64();

                if b == 0.0 {
                    return Err("Cannot divide by zero".into());
                }

                Ok(Value::Float(a / b).into())
            }

            // ...otherwise, all is assumed as integer.
            (a, b) => {
                let a = a.to_i64();
                let b = b.to_i64();

                if b == 0 {
                    return Err("Cannot divide by zero".into());
                }

                // If there's no remainder, perform an integer division
                if a % b == 0 {
                    Ok(Value::Integer(a / b).into())
                }
                // Otherwise a floating point division
                else {
                    Ok(Value::Float(a as f64 / b as f64).into())
                }
            }
        }
    }

    // Negation
    pub fn neg(&self) -> Result<RefValue, Error> {
        match &*self.borrow() {
            Value::Float(v) => Ok(Value::Float(-v).into()),
            _ => Ok(Value::Integer(-self.to_i64()).into()),
        }
    }

    // Logical not
    pub fn not(&self) -> Result<RefValue, Error> {
        Ok(if self.is_true() {
            Value::False
        } else {
            Value::True
        }
        .into())
    }
}

impl std::fmt::Display for RefValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::fmt::Debug for RefValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.borrow().fmt(f)
    }
}

impl From<Value> for RefValue {
    fn from(value: Value) -> Self {
        RefValue {
            value: Rc::new(RefCell::new(value)),
        }
    }
}

// Conversion from native types into Value

impl From<bool> for RefValue {
    fn from(value: bool) -> Self {
        if value {
            Value::True.into()
        } else {
            Value::False.into()
        }
    }
}

impl From<i64> for RefValue {
    fn from(value: i64) -> Self {
        Value::Integer(value).into()
    }
}

impl From<f64> for RefValue {
    fn from(value: f64) -> Self {
        Value::Float(value).into()
    }
}

impl From<usize> for RefValue {
    fn from(value: usize) -> Self {
        Value::Addr(value).into()
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
    Str(Str),        // str
    List(Box<List>), // list
    Dict(Box<Dict>), // dict

    // Callables
    Object(Box<dyn Object>),
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
            RefValue::from(Value::Dict(Box::new(dict)))
        }
    };

    ( [ $($value:tt),* ] ) => {
        {
            let mut list = List::new();
            $( list.push(value!($value).into()); )*
            RefValue::from(Value::List(Box::new(list)))
        }
    };

    ( $value:expr ) => {
        if let Some(value) = (&$value as &dyn std::any::Any).downcast_ref::<bool>()
        {
            if *value {
                RefValue::from(Value::True)
            }
            else {
                RefValue::from(Value::False)
            }
        }
        else
        if let Some(value) = (&$value as &dyn std::any::Any).downcast_ref::<f32>()
        {
            RefValue::from(Value::Float(*value as f64))
        }
        else
        if let Some(value) = (&$value as &dyn std::any::Any).downcast_ref::<f64>()
        {
            RefValue::from(Value::Float(*value))
        }
        else
        if let Some(value) = (&$value as &dyn std::any::Any).downcast_ref::<i32>()
        {
            RefValue::from(Value::Integer(*value as i64))
        }
        else
        if let Some(value) = (&$value as &dyn std::any::Any).downcast_ref::<i64>()
        {
            RefValue::from(Value::Integer(*value))
        }
        else
        if let Some(value) = (&$value as &dyn std::any::Any).downcast_ref::<usize>()
        {
            RefValue::from(Value::Addr(*value))
        }
        else {
            RefValue::from(Value::Str($value.to_string().into()))
        }
    }
}

impl Value {
    // Retrieve type name of a value
    pub fn id(&self) -> usize {
        match self {
            Self::Object(object) => object.id(),
            _ => self as *const Self as usize,
        }
    }

    // Retrieve type name of a value
    pub fn name(&self) -> &'static str {
        match self {
            Self::Void => "void",
            Self::Null => "null",
            Self::True => "true",
            Self::False => "false",
            Self::Integer(_) => "int",
            Self::Float(_) => "float",
            Self::Addr(_) => "addr",
            Self::Str(_) => "str",
            Self::List(_) => "list",
            Self::Dict(_) => "dict",
            Self::Object(object) => object.name(),
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
            Self::Str(s) => s.repr(),
            Self::List(l) => l.repr(),
            Self::Dict(d) => d.repr(),
            Self::Object(object) => object.repr(),
        }
    }

    /// Get a value's boolean meaning.
    pub fn is_true(&self) -> bool {
        match self {
            Self::Void | Self::Null | Self::False => false,
            Self::Integer(i) => *i != 0,
            Self::Float(f) => *f != 0.0,
            Self::Str(s) => s.len() > 0,
            Self::List(l) => l.len() > 0,
            Self::Dict(d) => d.len() > 0,
            _ => true, // everything else is just true as it exists.
        }
    }

    /// Get value's integer representation.
    pub fn to_i64(&self) -> i64 {
        match self {
            Self::True => 1,
            Self::Integer(i) => *i,
            Self::Float(f) => *f as i64,
            Self::Str(s) => {
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
            Self::Str(s) => {
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
            Self::Str(s) => {
                // todo: JavaScript-style parseInt-like behavior?
                match s.parse::<usize>() {
                    Ok(i) => i,
                    Err(_) => 0,
                }
            }
            _ => self as *const Self as usize,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Void => "".to_string(),
            Value::Str(s) => s.as_str().to_string(),
            _ => self.repr(),
        }
    }

    /// Retrieve &str from a value in case it is a string.
    pub fn str(&self) -> Option<&str> {
        if let Self::Str(s) = self {
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

    /// Check whether a value is object, and when its object if with or without arguments.
    pub fn is_callable(&self, with_arguments: bool) -> bool {
        if let Value::Object(object) = self {
            object.is_callable(with_arguments)
        } else {
            false
        }
    }

    /// Check whether a value is consuming
    pub fn is_consuming(&self) -> bool {
        if let Value::Object(object) = self {
            object.is_consuming()
        } else {
            false
        }
    }

    /// Check whether a value is consuming
    pub fn is_nullable(&self) -> bool {
        if let Value::Object(object) = self {
            object.is_nullable()
        } else {
            false
        }
    }
}

/// Convert a RefValue into a Value
impl From<RefValue> for Value {
    fn from(value: RefValue) -> Self {
        match Rc::try_unwrap(value.value) {
            Ok(value) => value.into_inner(),
            Err(value) => value.borrow().clone(),
        }
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
    assert_eq!(Value::Str("hello world".to_string()).to_string().unwrap(), Some("hello world".to_string()));
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

    let s = Value::Str("42".to_string());
    assert_eq!(s.to_string(), Some("42".to_string()));
    assert_eq!(s.to_i64(), Some(42));
    assert_eq!(s.to_f64(), Some(42.0));
    assert_eq!(s.to_string(), Some("42".to_string()));

    let s = Value::Str("gunshot42".to_string());
    assert_eq!(s.to_string(), Some("gunshot42".to_string()));
    assert_eq!(s.to_i64(), Some(0));
}

#[test]
fn test_add() {
    let x = Value::Integer(42);
    let y = Value::Str("hello".to_string());
    assert_eq!(x + y, Value::Str("42hello".to_string()));

    let x = Value::Integer(42);
    let y = Value::Str("hello".to_string());
    assert_eq!(y + x, Value::Str("hello42".to_string()));

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
    let y = Value::Str("hello".to_string());
    assert_eq!(x * y, Value::Str("hellohellohello".to_string()));

    assert_eq!(Value::Bool(true) * Value::Str("hello".to_string()), Value::Str("hello".to_string()));
    assert_eq!(Value::Bool(false) * Value::Str("hello".to_string()), Value::Str("".to_string()));

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
