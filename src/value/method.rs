//! A method object represents an object's method call (currently only for built-ins)

use super::{Callable, Dict, RefValue, Value};
use crate::vm::*;

#[derive(Debug, Clone)]
pub struct Method {
    pub(super) object: RefValue,
    pub(super) method: RefValue,
}

impl Callable for Method {
    fn name(&self) -> &str {
        "method"
    }

    fn is_callable(&self, with_arguments: bool) -> bool {
        self.method.borrow().is_callable(with_arguments)
    }

    fn is_consuming(&self) -> bool {
        self.method.borrow().is_consuming()
    }

    fn call(
        &self,
        context: &mut Context,
        args: usize,
        nargs: Option<Dict>,
    ) -> Result<Accept, Reject> {
        // A method call injects the relating "this" object into the stack and calls the method afterwards.
        context.runtime.stack.insert(
            context.runtime.stack.len() - args,
            Capture::Value(self.object.clone(), None, 0),
        );

        self.method.borrow().call(context, args + 1, nargs)
    }
}

impl From<Method> for Value {
    fn from(method: Method) -> Self {
        Value::Callable(Box::new(method))
    }
}
