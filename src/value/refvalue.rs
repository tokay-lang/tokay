use super::{BoxedObject, Dict, Method, Object, Value};
use crate::builtin::Builtin;
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

    pub fn binary_op(self, operand: RefValue, op: &str) -> Result<RefValue, String> {
        // Obtain tuple of name and severity
        let compare = {
            let this = self.borrow();
            let other = operand.borrow();

            (
                // Any severity of 0 falls back to int
                if this.severity() == 0 {
                    ("int", 0)
                } else {
                    (this.name(), this.severity())
                },
                // Any severity of 0 falls back to int
                if other.severity() == 0 {
                    ("int", 0)
                } else {
                    (other.name(), other.severity())
                },
            )
        };

        if compare.0 .1 > compare.1 .1 {
            let builtin = Builtin::get_method(compare.0 .0, op)?;
            Ok(builtin.call(None, vec![self, operand])?.unwrap())
        } else {
            let builtin = Builtin::get_method(compare.1 .0, op)?;
            Ok(builtin.call(None, vec![self, operand])?.unwrap())
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
