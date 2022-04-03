use super::{BoxedObject, Dict, Method, Object, Value};
use crate::builtin::Builtin;
use crate::value;
use crate::vm::{Accept, Context, Reject};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, PartialEq, PartialOrd)]
pub struct RefValue {
    value: Rc<RefCell<Value>>,
}

impl RefValue {
    /** Creates a callable Method object from a RefValue and a given method name. */
    pub fn create_method(&self, method_name: &str) -> Result<RefValue, String> {
        let builtin = Builtin::get_method(self.name(), method_name)?;
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
        let builtin = Builtin::get_method(self.name(), name)?;

        // Inject own value as first parameter.
        args.insert(0, self.clone());

        // Call the builtin directly.
        builtin.call(None, args)
    }

    pub fn unary_op(self, op: &str) -> Result<RefValue, String> {
        let name = {
            let this = &mut *self.borrow_mut();

            match this {
                Value::Object(this) => this.name(),
                Value::Addr(addr) => {
                    // addr fast lane iinc, idec
                    match op {
                        "iinc" => {
                            *addr += 1;
                            return Ok(self.clone());
                        }
                        "idec" => {
                            if *addr > 0 {
                                *addr -= 1;
                            }
                            return Ok(self.clone());
                        }
                        _ => "addr",
                    }
                }
                Value::Float(float) => {
                    // float fast lane neg, iinc, idec
                    match op {
                        "neg" => return Ok(value!(-*float)),
                        "iinc" => {
                            *float += 1.0;
                            return Ok(self.clone());
                        }
                        "idec" => {
                            *float -= 1.0;
                            return Ok(self.clone());
                        }
                        _ => "float",
                    }
                }
                Value::Int(int) => {
                    // int fast lane neg, iinc, idec
                    match op {
                        "neg" => return Ok(value!(-*int)),
                        "iinc" => {
                            *int += 1;
                            return Ok(self.clone());
                        }
                        "idec" => {
                            *int -= 1;
                            return Ok(self.clone());
                        }
                        _ => "int",
                    }
                }
                _ => "int",
            }
        };

        match Builtin::get_method(name, op) {
            Ok(builtin) => Ok(builtin.call(None, vec![self])?.unwrap()),
            // Default "not"
            Err(notfound) => match op {
                // default fallback for not
                "not" => Ok(value!(!self.is_true())),
                // default fallback for inline inc is an inline add by 1
                "iinc" if name == "int" => {
                    let ret = self.clone().binary_op(value!(1 as i64), "iadd")?;
                    *self.borrow_mut() = ret.into();
                    Ok(self)
                }
                // default fallback for inline dec is an inline sub by 1
                "idec" if name == "int" => {
                    let ret = self.clone().binary_op(value!(1 as i64), "isub")?;
                    *self.borrow_mut() = ret.into();
                    Ok(self)
                }
                _ => Err(notfound),
            },
        }
    }

    pub fn binary_op(self, operand: RefValue, op: &str) -> Result<RefValue, String> {
        let name = {
            // Operations starting with "i" are inline
            if op.starts_with("i") {
                // For fast-lane inline operations, the self must be borrowed mutable.
                let mut this = self.borrow_mut();

                // In case the operand cannot be borrowed, self and operand might be the same.
                if let Ok(that) = operand.try_borrow() {
                    match (&mut *this, &*that) {
                        // Object wins by severity.
                        (Value::Object(_), _) | (_, Value::Object(_)) => {
                            if that.severity() > this.severity() {
                                Some(that.name())
                            } else {
                                Some(this.name())
                            }
                        }

                        // Addr inline fast-lane
                        (Value::Addr(addr), _) => match op {
                            "iadd" => {
                                *addr += that.to_usize();
                                return Ok(self.clone());
                            }
                            "imul" => {
                                *addr *= that.to_usize();
                                return Ok(self.clone());
                            }
                            _ => None,
                        },

                        // Float inline fast-lane
                        (Value::Float(float), _) => match op {
                            "iadd" => {
                                *float += that.to_f64();
                                return Ok(self.clone());
                            }
                            "imul" => {
                                *float *= that.to_f64();
                                return Ok(self.clone());
                            }
                            "isub" => {
                                *float -= that.to_f64();
                                return Ok(self.clone());
                            }
                            _ => None,
                        },

                        // Int inline fast-lane
                        (Value::Int(int), _) => match op {
                            "iadd" => {
                                *int += that.to_i64();
                                return Ok(self.clone());
                            }
                            "imul" => {
                                *int *= that.to_i64();
                                return Ok(self.clone());
                            }
                            "isub" => {
                                *int -= that.to_i64();
                                return Ok(self.clone());
                            }
                            _ => None,
                        },

                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                let this = &*self.borrow();
                let that = &*operand.borrow();

                // Try to match comaprion operation
                match op {
                    "eq" => return Ok(value!(this == that)),
                    "neq" => return Ok(value!(this != that)),
                    "lt" => return Ok(value!(this < that)),
                    "lteq" => return Ok(value!(this <= that)),
                    "gt" => return Ok(value!(this > that)),
                    "gteq" => return Ok(value!(this >= that)),
                    _ => {}
                }

                // Try to match operation by type
                match (this, that) {
                    // Object wins by severity.
                    (Value::Object(_), _) | (_, Value::Object(_)) => {
                        if that.severity() > this.severity() {
                            Some(that.name())
                        } else {
                            Some(this.name())
                        }
                    }

                    // Addr
                    (Value::Addr(_), _) | (_, Value::Addr(_)) => match op {
                        "add" => return Ok(value!(this.to_usize() + that.to_usize())),
                        "mul" => return Ok(value!(this.to_usize() * that.to_usize())),
                        "sub" => {
                            let minuend = this.to_usize();
                            let subtrahend = that.to_usize();

                            if subtrahend > minuend {
                                return Err(String::from(
                                    "Attemt to substract with overflow (addr-value)",
                                ));
                            }

                            return Ok(value!(minuend - subtrahend));
                        }
                        "div" => {
                            let dividend = this.to_usize();
                            let divisor = that.to_usize();

                            if divisor == 0 {
                                return Err(String::from("Division by zero"));
                            }

                            // If there's no remainder, perform an integer division
                            if dividend % divisor == 0 {
                                return Ok(value!(dividend / divisor));
                            }
                            // Otherwise do a floating point division
                            else {
                                return Ok(value!(dividend as f64 / divisor as f64));
                            }
                        }
                        _ => None,
                    },
                    (Value::Float(_), _) | (_, Value::Float(_)) => match op {
                        "add" => return Ok(value!(this.to_f64() + that.to_f64())),
                        "mul" => return Ok(value!(this.to_f64() * that.to_f64())),
                        "sub" => return Ok(value!(this.to_f64() - that.to_f64())),
                        "div" => {
                            let dividend = this.to_f64();
                            let divisor = that.to_f64();

                            if divisor == 0.0 {
                                return Err(String::from("Division by zero"));
                            }

                            return Ok(value!(dividend / divisor));
                        }
                        _ => None,
                    },
                    (_, _) => match op {
                        "add" => return Ok(value!(this.to_i64() + that.to_i64())),
                        "mul" => return Ok(value!(this.to_i64() * that.to_i64())),
                        "sub" => return Ok(value!(this.to_i64() - that.to_i64())),
                        "div" => {
                            let dividend = this.to_i64();
                            let divisor = that.to_i64();

                            if divisor == 0 {
                                return Err(String::from("Division by zero"));
                            }

                            // If there's no remainder, perform an integer division
                            if dividend % divisor == 0 {
                                return Ok(value!(dividend / divisor));
                            }
                            // Otherwise do a floating point division
                            else {
                                return Ok(value!(dividend as f64 / divisor as f64));
                            }
                        }
                        _ => None,
                    },
                }
            }
        };

        // When a type name was emitted, try to call builtin-function for operation
        if let Some(name) = name {
            match Builtin::get_method(name, op) {
                Ok(builtin) => return Ok(builtin.call(None, vec![self, operand])?.unwrap()),
                // default "inline" operation is the non-inline operation assigning the result to itself
                Err(_) if op.starts_with("i") => {}
                Err(err) => return Err(err),
            }
        }

        // Perform expensive inline operation
        assert!(op.starts_with("i"));
        let res = self.clone().binary_op(operand, &op[1..])?;
        *self.borrow_mut() = res.into();
        Ok(self)
    }
}

impl Object for RefValue {
    fn id(&self) -> usize {
        self.borrow().id()
    }

    fn severity(&self) -> u8 {
        self.borrow().severity()
    }

    fn name(&self) -> &'static str {
        self.borrow().name()
    }

    fn repr(&self) -> String {
        self.borrow().repr()
    }

    fn is_void(&self) -> bool {
        matches!(&*self.borrow(), Value::Void)
    }

    fn is_true(&self) -> bool {
        self.borrow().is_true()
    }

    fn to_i64(&self) -> i64 {
        self.borrow().to_i64()
    }

    fn to_f64(&self) -> f64 {
        self.borrow().to_f64()
    }

    fn to_usize(&self) -> usize {
        self.borrow().to_usize()
    }

    fn to_string(&self) -> String {
        self.borrow().to_string()
    }

    fn is_callable(&self, with_arguments: bool) -> bool {
        self.borrow().is_callable(with_arguments)
    }

    fn is_consuming(&self) -> bool {
        self.borrow().is_consuming()
    }

    fn is_nullable(&self) -> bool {
        self.borrow().is_nullable()
    }

    fn call(
        &self,
        context: &mut Context,
        args: usize,
        nargs: Option<Dict>,
    ) -> Result<Accept, Reject> {
        self.borrow().call(context, args, nargs)
    }
}

impl std::ops::Deref for RefValue {
    type Target = Rc<RefCell<Value>>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl std::ops::DerefMut for RefValue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

/*
impl std::fmt::Display for RefValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.borrow().repr())
    }
}
*/

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

impl From<RefValue> for Value {
    fn from(value: RefValue) -> Self {
        match std::rc::Rc::try_unwrap(value.value) {
            Ok(value) => value.into_inner(),
            Err(value) => value.borrow().clone(),
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
