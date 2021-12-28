//! List object
use linkme::distributed_slice;

use super::{Object, RefValue, Value};
use crate::builtin::{Builtin, BUILTINS};
use crate::vm::*;

pub type List = Vec<RefValue>;

impl From<Value> for List {
    fn from(value: Value) -> Self {
        if let Value::List(list) = value {
            *list
        } else {
            vec![value.into()]
        }
    }
}

impl From<&Value> for List {
    fn from(value: &Value) -> Self {
        if let Value::List(list) = value {
            *list.clone()
        } else {
            vec![value.clone().into()]
        }
    }
}

impl Object for List {
    fn name(&self) -> &str {
        "list"
    }

    fn repr(&self) -> String {
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

    /*
    fn get_index(&self, index: &Value) -> Result<RefValue, String> {
        let index = index.to_addr();
        if let Some(value) = self.get(index) {
            Ok(value.clone())
        } else {
            Err(format!("Index {} out of bounds", index))
        }
    }

    fn set_index(&mut self, index: &Value, value: RefValue) -> Result<(), String> {
        let index = index.to_addr();
        if index < self.len() {
            self[index] = value;
            Ok(())
        } else if index == self.len() {
            self.push(value);
            Ok(())
        } else {
            Err(format!("Index {} out of bounds", index))
        }
    }
    */
}

#[distributed_slice(BUILTINS)]
static LIST: Builtin = Builtin {
    name: "list",
    signature: "",
    func: |_context, _args| {
        // fixme: Incomplete, concept missing.
        Ok(Accept::Push(Capture::Value(
            Value::List(Box::new(List::new())).into(),
            None,
            10,
        )))
    },
};

/*
#[distributed_slice(BUILTINS)]
static LIST_PUSH: Builtin = Builtin {
    name: "list_push",
    signature: "self item",
    func: |_context, mut args| {
        let mut list = args.remove(0).unwrap();
        let item = args.remove(0).unwrap();

        // If list is not a list, turn it into a list
        if list.borrow().list().is_none() {
            let new = list.borrow().to_list();
            list = Value::List(Box::new(new)).into();
        }

        // Push the item to the list
        if let Value::List(list) = &mut *list.borrow_mut() {
            list.push(item);
        }

        Ok(Accept::Push(Capture::Value(list, None, 10)))
    },
};
*/
