//! Numeric object
use super::{BoxedObject, Object, RefValue, Value};
use crate::error::Error;
use macros::tokay_method;

/// Numeric object type
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Numeric {
    Int(i64),
    Float(f64),
    Addr(usize),
}

impl Object for Numeric {
    fn name(&self) -> &'static str {
        match self {
            Self::Int(_) => "int",
            Self::Float(_) => "float",
            Self::Addr(_) => "addr",
        }
    }

    fn repr(&self) -> String {
        match self {
            Self::Int(i) => format!("{}", i),
            Self::Float(f) => format!("{}", f),
            Self::Addr(a) => format!("{}", a),
        }
    }

    fn is_true(&self) -> bool {
        match self {
            Self::Int(i) => *i != 0,
            Self::Float(f) => *f != 0.0,
            Self::Addr(a) => *a != 0,
        }
    }

    fn to_i64(&self) -> i64 {
        match self {
            Self::Int(i) => *i,
            Self::Float(f) => *f as i64,
            Self::Addr(a) => *a as i64,
        }
    }

    fn to_f64(&self) -> f64 {
        match self {
            Self::Int(i) => *i as f64,
            Self::Float(f) => *f,
            Self::Addr(a) => *a as f64,
        }
    }

    fn to_usize(&self) -> usize {
        match self {
            Self::Int(i) => *i as usize,
            Self::Float(f) => *f as usize,
            Self::Addr(a) => *a,
        }
    }

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
}

impl Numeric {
    tokay_method!("int_new(int)", Ok(RefValue::from(int.to_i64())));
    tokay_method!("float_new(float)", Ok(RefValue::from(float.to_f64())));
    tokay_method!("addr_new(addr)", Ok(RefValue::from(addr.to_usize())));
}

impl From<i64> for RefValue {
    fn from(int: i64) -> Self {
        RefValue::from(Box::new(Numeric::Int(int)) as BoxedObject)
    }
}

impl From<i32> for RefValue {
    fn from(int: i32) -> Self {
        RefValue::from(int as i64)
    }
}

impl From<f64> for RefValue {
    fn from(float: f64) -> Self {
        RefValue::from(Box::new(Numeric::Float(float)) as BoxedObject)
    }
}

impl From<f32> for RefValue {
    fn from(float: f32) -> Self {
        RefValue::from(float as f64)
    }
}

impl From<usize> for RefValue {
    fn from(addr: usize) -> Self {
        RefValue::from(Box::new(Numeric::Addr(addr)) as BoxedObject)
    }
}
