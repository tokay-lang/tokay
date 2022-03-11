//! Tokay value
use std::any::Any;

use super::{BoxedObject, Dict, Object, RefValue};
use crate::vm::{Accept, Context, Reject};
use macros::tokay_method;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    // Atomics
    Void,  // void
    Null,  // null
    True,  // true
    False, // false

    // Numerics
    Int(i64),
    Float(f64),
    Addr(usize),

    // Objects
    Object(BoxedObject),
}

impl Value {
    /// Return reference to object of type T.
    pub fn object<T: Any>(&self) -> Option<&T> {
        if let Self::Object(o) = self {
            return o.as_any().downcast_ref::<T>();
        }

        None
    }

    /// Return mutable reference to object of type T.
    pub fn object_mut<T: Any>(&mut self) -> Option<&mut T> {
        if let Self::Object(o) = self {
            return o.as_any_mut().downcast_mut::<T>();
        }

        None
    }

    /// Extract object of type T from Val.
    pub fn object_into<T: Any>(self) -> Option<T> {
        if let Self::Object(o) = self {
            if let Ok(inner) = o.into_any().downcast::<T>() {
                return Some(*inner);
            }
        }

        None
    }

    tokay_method!("bool_new(b)", Ok(RefValue::from(b.is_true())));
    tokay_method!("int_new(i)", Ok(RefValue::from(i.to_i64())));
    tokay_method!("float_new(f)", Ok(RefValue::from(f.to_f64())));
    tokay_method!("addr_new(a)", Ok(RefValue::from(a.to_usize())));
}

impl Object for Value {
    fn severity(&self) -> u8 {
        match self {
            Self::Object(o) => o.severity(),
            _ => 0,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Void => "void",
            Self::Null => "null",
            Self::True => "true",
            Self::False => "false",
            Self::Int(_) => "int",
            Self::Float(_) => "float",
            Self::Addr(_) => "addr",
            Self::Object(o) => o.name(),
        }
    }

    fn repr(&self) -> String {
        match self {
            Self::Int(i) => format!("{}", i),
            Self::Float(f) => format!("{}", f),
            Self::Addr(a) => format!("{}", a),
            Self::Object(o) => o.repr(),
            _ => self.name().to_string(),
        }
    }

    fn is_void(&self) -> bool {
        matches!(self, Value::Void)
    }

    fn is_true(&self) -> bool {
        match self {
            Self::True => true,
            Self::Int(i) => *i != 0,
            Self::Float(f) => *f != 0.0,
            Self::Addr(a) => *a != 0,
            Self::Object(o) => o.is_true(),
            _ => false,
        }
    }

    fn to_i64(&self) -> i64 {
        match self {
            Self::True => 1,
            Self::Int(i) => *i,
            Self::Float(f) => *f as i64,
            Self::Addr(a) => *a as i64,
            Self::Object(o) => o.to_i64(),
            _ => 0,
        }
    }

    fn to_f64(&self) -> f64 {
        match self {
            Self::True => 1.0,
            Self::Int(i) => *i as f64,
            Self::Float(f) => *f,
            Self::Addr(a) => *a as f64,
            Self::Object(o) => o.to_f64(),
            _ => 0.0,
        }
    }

    fn to_usize(&self) -> usize {
        match self {
            Self::True => 1,
            Self::Int(i) => *i as usize,
            Self::Float(f) => *f as usize,
            Self::Addr(a) => *a,
            Self::Object(o) => o.to_usize(),
            _ => 0,
        }
    }

    fn to_string(&self) -> String {
        match self {
            Self::Void => "".to_string(),
            Self::Object(o) => o.to_string(),
            _ => self.repr(),
        }
    }

    fn is_callable(&self, with_arguments: bool) -> bool {
        if let Self::Object(object) = self {
            object.is_callable(with_arguments)
        } else {
            false
        }
    }

    fn is_consuming(&self) -> bool {
        if let Self::Object(object) = self {
            object.is_consuming()
        } else {
            false
        }
    }

    fn is_nullable(&self) -> bool {
        if let Self::Object(object) = self {
            object.is_nullable()
        } else {
            false
        }
    }

    fn call(
        &self,
        context: &mut Context,
        args: usize,
        nargs: Option<Dict>,
    ) -> Result<Accept, Reject> {
        if let Value::Object(object) = self {
            object.call(context, args, nargs)
        } else {
            Err(format!("'{}' cannot be called", self.repr()).into())
        }
    }

    /*
    fn unary_op(&self, op: char) -> Result<RefValue, Error> {
        match op {
            '!' => Ok(RefValue::from(!self.is_true())),
            '-' => match self {
                Self::Int(i) => Ok(RefValue::from(-i)),
                Self::Float(f) => Ok(RefValue::from(-f)),
                Self::Addr(_) => self.unsupported_unary_op(op),
            },
            op => self.unsupported_unary_op(op),
        }
    }

    /// Unary op
    fn binary_op(&self, op: char, a: &Value, b: &Value) -> Result<RefValue, Error> {
        println!("{:?} {} {:?}", a, op, b);
        match op {
            '+' => match self {
                Self::Int(_) => Ok(RefValue::from(a.to_i64() + b.to_i64())),
                Self::Float(_) => Ok(RefValue::from(a.to_f64() + b.to_f64())),
                Self::Addr(_) => Ok(RefValue::from(a.to_usize() + b.to_usize())),
            },
            '-' => match self {
                Self::Int(_) => Ok(RefValue::from(a.to_i64() - b.to_i64())),
                Self::Float(_) => Ok(RefValue::from(a.to_f64() - b.to_f64())),
                Self::Addr(_) => Ok(RefValue::from(a.to_usize() - b.to_usize())),
            },
            '*' => match self {
                Self::Int(_) => Ok(RefValue::from(a.to_i64() * b.to_i64())),
                Self::Float(_) => Ok(RefValue::from(a.to_f64() * b.to_f64())),
                Self::Addr(_) => Ok(RefValue::from(a.to_usize() * b.to_usize())),
            },
            '/' => match self {
                Self::Int(_) => {
                    let dividend = a.to_i64();
                    let divisor = b.to_i64();

                    if divisor == 0 {
                        return Err(Error::from("Division by zero"));
                    }

                    // If there's no remainder, perform an integer division
                    if dividend % divisor == 0 {
                        Ok(RefValue::from(dividend / divisor))
                    }
                    // Otherwise do a floating point division
                    else {
                        Ok(RefValue::from(dividend as f64 / divisor as f64))
                    }
                }
                Self::Float(_) => {
                    let dividend = a.to_f64();
                    let divisor = b.to_f64();

                    if divisor == 0.0 {
                        return Err(Error::from("Division by zero"));
                    }

                    Ok(RefValue::from(dividend / divisor))
                }
                Self::Addr(_) => {
                    // todo...
                    let dividend = a.to_usize();
                    let divisor = b.to_usize();

                    if divisor == 0 {
                        return Err(Error::from("Division by zero"));
                    }

                    Ok(RefValue::from(dividend / divisor))
                }
            },

            op => self.unsupported_binary_op(op, a, b),
        }
    }
    */
}

impl From<bool> for RefValue {
    fn from(value: bool) -> Self {
        RefValue::from(if value { Value::True } else { Value::False })
    }
}

impl From<i64> for RefValue {
    fn from(int: i64) -> Self {
        RefValue::from(Value::Int(int))
    }
}

impl From<i32> for RefValue {
    fn from(int: i32) -> Self {
        RefValue::from(int as i64)
    }
}

impl From<f64> for RefValue {
    fn from(float: f64) -> Self {
        RefValue::from(Value::Float(float))
    }
}

impl From<f32> for RefValue {
    fn from(float: f32) -> Self {
        RefValue::from(float as f64)
    }
}

impl From<usize> for RefValue {
    fn from(addr: usize) -> Self {
        RefValue::from(Value::Addr(addr))
    }
}
