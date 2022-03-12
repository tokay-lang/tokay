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

    fn call_binary_method(self, operand: RefValue, name: &str) -> Result<RefValue, String> {
        if self.severity() > operand.severity() {
            let builtin = self.get_method(name)?;
            Ok(builtin.call(None, vec![self, operand])?.unwrap())
        } else {
            let builtin = operand.get_method(name)?;
            Ok(builtin.call(None, vec![self, operand])?.unwrap())
        }
    }

    // Addition
    pub fn binary_add(self, operand: RefValue) -> Result<RefValue, String> {
        let augend = &*self.borrow();
        let addend = &*operand.borrow();

        Ok(match (augend, addend) {
            // Have an object? Let's decide by precedence.
            (Value::Object(_), _) | (_, Value::Object(_)) => {
                self.clone().call_binary_method(operand.clone(), "add")?
            }
            // Fallback for any basic types.
            (Value::Float(_), _) | (_, Value::Float(_)) => {
                value!(augend.to_f64() + addend.to_f64())
            }
            (Value::Addr(_), _) | (_, Value::Addr(_)) => {
                value!(augend.to_usize() + addend.to_usize())
            }
            _ => {
                value!(augend.to_i64() + addend.to_i64())
            }
        })
    }

    // Subtraction
    pub fn binary_sub(self, operand: RefValue) -> Result<RefValue, String> {
        let minuend = &*self.borrow();
        let subtrahend = &*operand.borrow();

        Ok(match (minuend, subtrahend) {
            // Have an object? Let's decide by precedence.
            (Value::Object(_), _) | (_, Value::Object(_)) => {
                self.clone().call_binary_method(operand.clone(), "sub")?
            }
            // Fallback for any basic types.
            (Value::Float(_), _) | (_, Value::Float(_)) => {
                value!(minuend.to_f64() - subtrahend.to_f64())
            }
            (Value::Addr(_), _) | (_, Value::Addr(_)) => {
                let minuend = minuend.to_usize();
                let subtrahend = subtrahend.to_usize();

                if subtrahend > minuend {
                    return Err(String::from(
                        "Attemt to substract with overflow (addr-value)",
                    ));
                }

                value!(minuend - subtrahend)
            }
            _ => {
                value!(minuend.to_i64() - subtrahend.to_i64())
            }
        })
    }

    // Multiplication
    pub fn binary_mul(self, operand: RefValue) -> Result<RefValue, String> {
        let multiplier = &*self.borrow();
        let multiplicant = &*operand.borrow();

        Ok(match (multiplier, multiplicant) {
            // Have an object? Let's decide by precedence.
            (Value::Object(_), _) | (_, Value::Object(_)) => {
                self.clone().call_binary_method(operand.clone(), "mul")?
            }
            // Fallback for any basic types.
            (Value::Float(_), _) | (_, Value::Float(_)) => {
                value!(multiplier.to_f64() * multiplicant.to_f64())
            }
            (Value::Addr(_), _) | (_, Value::Addr(_)) => {
                value!(multiplier.to_usize() * multiplicant.to_usize())
            }
            _ => {
                value!(multiplier.to_i64() * multiplicant.to_i64())
            }
        })
    }

    // Division
    pub fn binary_div(self, operand: RefValue) -> Result<RefValue, String> {
        let dividend = &*self.borrow();
        let divisor = &*operand.borrow();

        Ok(match (dividend, divisor) {
            // Have an object? Let's decide by precedence.
            (Value::Object(_), _) | (_, Value::Object(_)) => {
                self.clone().call_binary_method(operand.clone(), "div")?
            }
            // Fallback for any basic types.
            (Value::Float(_), _) | (_, Value::Float(_)) => {
                let dividend = dividend.to_f64();
                let divisor = divisor.to_f64();

                if divisor == 0.0 {
                    return Err(String::from("Division by zero"));
                }

                value!(dividend / divisor)
            }
            (Value::Addr(_), _) | (_, Value::Addr(_)) => {
                let dividend = dividend.to_usize();
                let divisor = divisor.to_usize();

                if divisor == 0 {
                    return Err(String::from("Division by zero"));
                }

                // If there's no remainder, perform an integer division
                if dividend % divisor == 0 {
                    value!(dividend / divisor)
                }
                // Otherwise do a floating point division
                else {
                    value!(dividend as f64 / divisor as f64)
                }
            }
            _ => {
                let dividend = dividend.to_i64();
                let divisor = divisor.to_i64();

                if divisor == 0 {
                    return Err(String::from("Division by zero"));
                }

                // If there's no remainder, perform an integer division
                if dividend % divisor == 0 {
                    value!(dividend / divisor)
                }
                // Otherwise do a floating point division
                else {
                    value!(dividend as f64 / divisor as f64)
                }
            }
        })
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
