//! Tokay value
use super::{BoxedObject, Dict, Object, RefValue};
use crate::vm::{Accept, Context, Reject};
use macros::tokay_method;
use std::any::Any;
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
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

    // Constructors
    tokay_method!("bool(value)", Ok(RefValue::from(value.is_true())));
    tokay_method!("int(value)", Ok(RefValue::from(value.to_i64())));
    tokay_method!("float(value)", Ok(RefValue::from(value.to_f64())));
    tokay_method!("addr(value)", Ok(RefValue::from(value.to_usize())));
}

impl Object for Value {
    fn severity(&self) -> u8 {
        match self {
            Self::Int(_) => 1,
            Self::Float(_) => 2,
            Self::Addr(_) => 3,
            Self::Object(o) => o.severity(),
            _ => 0,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Void => "void",
            Self::Null => "null",
            Self::True | Self::False => "bool",
            Self::Int(_) => "int",
            Self::Float(_) => "float",
            Self::Addr(_) => "addr",
            Self::Object(o) => o.name(),
        }
    }

    fn repr(&self) -> String {
        match self {
            Self::True => "true".to_string(),
            Self::False => "false".to_string(),
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

    fn is_callable(&self, without_arguments: bool) -> bool {
        if let Self::Object(object) = self {
            object.is_callable(without_arguments)
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
            Err(format!("'{}' object is not callable", self.name()).into())
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Object(i), Self::Object(j)) => i.partial_cmp(j),
            (Self::Object(_), _) => Some(Ordering::Greater),
            (_, Self::Object(_)) => Some(Ordering::Less),

            (Self::Addr(i), j) => i.partial_cmp(&j.to_usize()),
            (i, Self::Addr(j)) => i.to_usize().partial_cmp(j),

            (Self::Float(i), j) => i.partial_cmp(&j.to_f64()),
            (i, Self::Float(j)) => i.to_f64().partial_cmp(j),

            (Self::Int(i), j) => i.partial_cmp(&j.to_i64()),
            (i, j) => i.to_i64().partial_cmp(&j.to_i64()),
        }
    }
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
