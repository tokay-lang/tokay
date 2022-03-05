//! Tokay value and object representation
use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::builtin::Builtin;
use crate::error::Error;
use crate::vm::{Accept, Context, Reject};

pub mod dict;
pub mod list;
mod method;
pub mod numeric;
mod object;
mod parselet;
pub mod str;
pub mod token;

pub use self::str::Str;
pub use dict::Dict;
pub use list::List;
pub use method::Method;
pub use numeric::Numeric;
pub use object::{BoxedObject, Object};
pub use parselet::{Parselet, ParseletRef};
pub use token::Token;

/** Value construction helper-macro

This macro is used to easily construct Tokay values in Rust code.

Examples:
```
use tokay::value;

let i = value!(1);
let s = value!("String");
let l = value!([1, 2, 3]);
let d = value!(["a" => 1, "b" => 2, "c" => 3]);
```
*/
#[macro_export]
macro_rules! value {
    ( [ $($key:literal => $value:tt),* ] ) => {
        {
            let mut dict = $crate::value::Dict::new();
            $( dict.insert($key.to_string(), value!($value)); )*
            $crate::RefValue::from(dict)
        }
    };

    ( [ $($value:tt),* ] ) => {
        {
            let mut list = $crate::value::List::new();
            $( list.push(value!($value)); )*
            $crate::RefValue::from(list)
        }
    };

    ( void ) => {
        $crate::RefValue::from($crate::value::Value::Void)
    };

    ( null ) => {
        $crate::RefValue::from($crate::value::Value::Null)
    };

    ( $value:expr ) => {
        $crate::RefValue::from($value)
    }
}

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
        //self.is("void")
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

    pub fn unary_op(&self, op: char) -> Result<RefValue, Error> {
        self.borrow().unary_op(op)
    }

    pub fn binary_op(&self, op: char, operand: RefValue) -> Result<RefValue, Error> {
        self.borrow().binary_op(op, &*operand.borrow())
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

impl From<BoxedObject> for RefValue {
    fn from(value: BoxedObject) -> Self {
        RefValue {
            value: Rc::new(RefCell::new(Value::Object(value))),
        }
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

    // Objects
    Object(BoxedObject),
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
            Self::Object(object) => object.repr(),
        }
    }

    /// Get a value's boolean meaning.
    pub fn is_true(&self) -> bool {
        match self {
            Self::True => true,
            Self::Object(obj) => obj.is_true(),
            _ => false,
        }
    }

    /// Get value's integer representation.
    pub fn to_i64(&self) -> i64 {
        match self {
            Self::True => 1,
            Self::Object(obj) => obj.to_i64(),
            _ => 0,
        }
    }

    /// Get value's float representation.
    pub fn to_f64(&self) -> f64 {
        match self {
            Self::True => 1.0,
            Self::Object(obj) => obj.to_f64(),
            _ => 0.0,
        }
    }

    /// Get value's usize representation.
    pub fn to_usize(&self) -> usize {
        match self {
            Self::True => 1 as usize,
            Self::Object(obj) => obj.to_usize(),
            _ => 0 as usize,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Void => "".to_string(),
            Value::Object(obj) => obj.to_string(),
            _ => self.repr(),
        }
    }

    /// Return reference to object of type T.
    pub fn object<T: Any>(&self) -> Option<&T> {
        if let Value::Object(obj) = self {
            return obj.as_any().downcast_ref::<T>();
        }

        None
    }

    /// Return mutable reference to object of type T.
    pub fn object_mut<T: Any>(&mut self) -> Option<&mut T> {
        if let Value::Object(obj) = self {
            return obj.as_any_mut().downcast_mut::<T>();
        }

        None
    }

    /// Extract object of type T from Val.
    pub fn object_into<T: Any>(self) -> Option<T> {
        if let Value::Object(obj) = self {
            if let Ok(inner) = obj.into_any().downcast::<T>() {
                return Some(*inner);
            }
        }

        None
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

    pub fn unary_op(&self, op: char) -> Result<RefValue, Error> {
        if let Value::Object(obj) = self {
            obj.unary_op(op)
        } else {
            value!(self.to_i64()).borrow().unary_op(op) // todo: Temporary.
        }
    }

    pub fn binary_op(&self, op: char, operand: &Value) -> Result<RefValue, Error> {
        match (self, operand) {
            (Value::Object(a), Value::Object(b)) => {
                // Check for operand precedence
                if a.severity() < b.severity() {
                    b.binary_op(op, self, operand)
                // If equal, use first operand's type
                } else {
                    a.binary_op(op, self, operand)
                }
            }
            (Value::Object(obj), _) |  (_, Value::Object(obj)) => obj.binary_op(op, self, operand),
            _ => {
                let value = value!(self.to_i64()); // todo: Temporary.
                let value = value.borrow();
                value
                    .object::<Numeric>()
                    .unwrap()
                    .binary_op(op, self, operand)
            }
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

// Conversion from native types into Value

impl From<bool> for RefValue {
    fn from(value: bool) -> Self {
        RefValue::from(if value { Value::True } else { Value::False })
    }
}
