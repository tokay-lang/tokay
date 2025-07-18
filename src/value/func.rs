use crate::{Accept, Context, Dict, Object, RefValue, Reject, Value};
use std::rc::Rc;

// Better abstraction of a built-in function
#[derive(Clone)]
pub struct Func {
    pub func: Rc<
        dyn Fn(Option<&mut Context>, Vec<RefValue>, Option<Dict>) -> Result<Accept, Reject>
            + Send
            + Sync,
    >, // Function
}

impl Object for Func {
    fn name(&self) -> &'static str {
        "func"
    }

    fn repr(&self) -> String {
        //self.0.name.to_string()
        "func".to_string()
    }

    fn is_callable(&self, _without_arguments: bool) -> bool {
        true // Always callable, arguments are being checked by the function.
    }

    fn is_consuming(&self) -> bool {
        false
        //crate::utils::identifier_is_consumable(self.0.name)
    }

    fn call(
        &self,
        context: Option<&mut Context>,
        args: Vec<RefValue>,
        nargs: Option<Dict>,
    ) -> Result<Accept, Reject> {
        (self.func)(context, args, nargs)
    }

    fn call_direct(
        &self,
        context: &mut Context,
        args: usize,
        nargs: Option<Dict>,
    ) -> Result<Accept, Reject> {
        let args = context.drain(args);
        (self.func)(Some(context), args, nargs)
    }
}

impl PartialEq for Func {
    fn eq(&self, other: &Self) -> bool {
        //self.0.name == other.0.name
        false
    }
}

impl PartialOrd for Func {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        //self.0.name.partial_cmp(&other.0.name)
        None
    }
}

impl std::fmt::Debug for Func {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //write!(f, "{}", self.0.name)
        write!(f, "func")
    }
}

impl From<Func> for RefValue {
    fn from(func: Func) -> Self {
        Value::Object(Box::new(func)).into()
    }
}
