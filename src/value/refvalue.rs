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
            let this = &mut *self.borrow_mut();
            let that = &*operand.borrow();

            //println!("{} {:?} {:?}", op, this, that);

            match (this, that) {
                (Value::Object(this), that) => {
                    if that.severity() > this.severity() {
                        that.name()
                    } else {
                        this.name()
                    }
                }
                (this, Value::Object(that)) => {
                    if that.severity() > this.severity() {
                        that.name()
                    } else {
                        this.name()
                    }
                }
                (Value::Addr(addr), _) => match op {
                    // addr fast lane add, iadd, mul, imul
                    "add" => return Ok(value!(*addr + that.to_usize())),
                    "iadd" => {
                        *addr += that.to_usize();
                        return Ok(self.clone());
                    }
                    "mul" => return Ok(value!(*addr * that.to_usize())),
                    "imul" => {
                        *addr *= that.to_usize();
                        return Ok(self.clone());
                    }
                    _ => "addr",
                },
                (_, Value::Addr(_)) => "addr",
                (Value::Float(float), _) => match op {
                    // float fast lane add, iadd, mul, imul
                    "add" => return Ok(value!(*float + that.to_f64())),
                    "iadd" => {
                        *float += that.to_f64();
                        return Ok(self.clone());
                    }
                    "mul" => return Ok(value!(*float * that.to_f64())),
                    "imul" => {
                        *float *= that.to_f64();
                        return Ok(self.clone());
                    }
                    _ => "float",
                },
                (_, Value::Float(_)) => "float",
                (Value::Int(int), _) => match op {
                    // int fast lane add, iadd, mul, imul
                    "add" => return Ok(value!(*int + that.to_i64())),
                    "iadd" => {
                        *int += that.to_i64();
                        return Ok(self.clone());
                    }
                    "mul" => return Ok(value!(*int * that.to_i64())),
                    "imul" => {
                        *int *= that.to_i64();
                        return Ok(self.clone());
                    }
                    _ => "int",
                },
                _ => "int",
            }
        };

        match Builtin::get_method(name, op) {
            Ok(builtin) => Ok(builtin.call(None, vec![self, operand])?.unwrap()),
            // default "inline" operation is the non-inline operating assigning the result to itself
            Err(_) if op.starts_with("i") => {
                let res = self.clone().binary_op(operand, &op[1..])?;
                *self.borrow_mut() = res.into();
                return Ok(self);
            }
            Err(err) => Err(err),
        }
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
