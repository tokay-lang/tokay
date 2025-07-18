use crate::{Accept, Context, Dict, Object, RefValue, Reject, Value};

// Better abstraction of a built-in function
pub struct Func {
    pub name: &'static str,
    pub func: Box<
        dyn Fn(Option<&mut Context>, Vec<RefValue>, Option<Dict>) -> Result<Accept, Reject>
            + Send
            + Sync,
    >,
}

#[derive(Clone)]
pub struct FuncRef(std::rc::Rc<Func>);

impl Object for FuncRef {
    fn name(&self) -> &'static str {
        self.0.name
    }

    fn repr(&self) -> String {
        self.0.name.to_string()
    }

    fn is_callable(&self, _without_arguments: bool) -> bool {
        true // Always callable, arguments are being checked by the function.
    }

    fn is_consuming(&self) -> bool {
        crate::utils::identifier_is_consumable(self.0.name)
    }

    fn call(
        &self,
        context: Option<&mut Context>,
        args: Vec<RefValue>,
        nargs: Option<Dict>,
    ) -> Result<Accept, Reject> {
        (self.0.func)(context, args, nargs)
    }

    fn call_direct(
        &self,
        context: &mut Context,
        args: usize,
        nargs: Option<Dict>,
    ) -> Result<Accept, Reject> {
        let args = context.drain(args);
        (self.0.func)(Some(context), args, nargs)
    }
}

impl PartialEq for FuncRef {
    fn eq(&self, other: &Self) -> bool {
        self.0.name == other.0.name
    }
}

impl PartialOrd for FuncRef {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.name.partial_cmp(&other.0.name)
    }
}

impl std::fmt::Debug for FuncRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.name)
    }
}

impl From<Func> for RefValue {
    fn from(func: Func) -> Self {
        Value::Object(Box::new(FuncRef(std::rc::Rc::new(func)))).into()
    }
}
