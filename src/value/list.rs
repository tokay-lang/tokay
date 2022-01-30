//! List object
use linkme::distributed_slice;
use macros::tokay_method;

use super::{RefValue, Value};
use crate::builtin::{Builtin, BUILTINS};
use crate::vm::*;

/// Alias for the inner list definition
type InnerList = Vec<RefValue>;

/// List object type
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct List {
    list: InnerList,
}

impl List {
    pub fn new() -> Self {
        Self {
            list: InnerList::new(),
        }
    }

    //#[tokay_method(name = "list_new")]
    #[tokay_method("list_new")]
    pub fn constructor(
        _context: Option<&mut Context>,
        mut args: Vec<Option<RefValue>>,
    ) -> Result<Accept, Reject> {
        let list = if args.len() == 1 {
            List::from(args.remove(0).unwrap())
        } else {
            List {
                list: args.into_iter().map(|item| item.unwrap()).collect(),
            }
        };

        Ok(Accept::Push(Capture::Value(list.into(), None, 10)))
    }

    #[tokay_method]
    pub fn list_push(
        _context: Option<&mut Context>,
        mut args: Vec<Option<RefValue>>,
    ) -> Result<Accept, Reject> {
        let mut list = args.remove(0).unwrap();
        let item = args.remove(0).unwrap();

        // If list is not a list, turn it into a list
        if !list.is("list") {
            list = Builtin::get("list")
                .unwrap()
                .call(None, vec![list])
                .unwrap()
                .unwrap();
        }

        // Push the item to the list
        if let Value::List(list) = &mut *list.borrow_mut() {
            list.push(item);
        }

        Ok(Accept::Push(Capture::Value(list, None, 10)))
    }

    pub fn repr(&self) -> String {
        let mut ret = "(".to_string();
        for item in self.iter() {
            if ret.len() > 1 {
                ret.push_str(", ");
            }

            ret.push_str(&item.borrow().repr());
        }

        if self.len() == 1 {
            ret.push_str(", ");
        }

        ret.push(')');
        ret
    }
}

impl std::ops::Deref for List {
    type Target = InnerList;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl std::ops::DerefMut for List {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}

impl std::iter::IntoIterator for List {
    type Item = RefValue;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.list.into_iter()
    }
}

impl From<Value> for List {
    fn from(value: Value) -> Self {
        if let Value::List(list) = value {
            *list
        } else {
            Self {
                list: vec![value.into()],
            }
        }
    }
}

impl From<&Value> for List {
    fn from(value: &Value) -> Self {
        if let Value::List(list) = value {
            *list.clone()
        } else {
            Self {
                list: vec![value.clone().into()],
            }
        }
    }
}

impl From<RefValue> for List {
    fn from(refvalue: RefValue) -> Self {
        if let Value::List(list) = &*refvalue.borrow() {
            *list.clone()
        } else {
            Self {
                list: vec![refvalue.clone()],
            }
        }
    }
}

/*
// fixme: This could be a replacement for value.list() but its usage is ugly.
impl<'list> From<&'list Value> for Option<&'list List> {
    fn from(value: &'list Value) -> Self {
        if let Value::List(list) = value {
            Some(&list)
        } else {
            None
        }
    }
}
*/

impl From<List> for RefValue {
    fn from(value: List) -> Self {
        Value::List(Box::new(value)).into()
    }
}

#[distributed_slice(BUILTINS)]
static LIST: Builtin = Builtin {
    name: "list",
    signature: "?",
    func: List::constructor,
};

#[distributed_slice(BUILTINS)]
static LIST_PUSH: Builtin = Builtin {
    name: "list_push",
    signature: "self item",
    func: List::list_push,
};
