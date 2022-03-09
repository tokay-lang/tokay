//! Tokay value and object representation
use std::cell::RefCell;
use std::rc::Rc;

use crate::builtin::Builtin;
use crate::vm::{Accept, Context, Reject};

pub mod dict;
pub mod list;
mod method;
mod object;
mod parselet;
pub mod str;
pub mod token;
pub mod value;

pub use self::str::Str;
pub use dict::Dict;
pub use list::List;
pub use method::Method;
pub use object::{BoxedObject, Object};
pub use parselet::{Parselet, ParseletRef};
pub use token::Token;
pub use value::Value;

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

impl From<BoxedObject> for RefValue {
    fn from(value: BoxedObject) -> Self {
        RefValue {
            value: Rc::new(RefCell::new(Value::Object(value))),
        }
    }
}
