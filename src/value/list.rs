//! List object

use super::{Object, RefValue, Value};

pub type List = Vec<RefValue>;

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

    fn is_true(&self) -> bool {
        self.len() > 0
    }

    fn is_list(&self) -> Option<&List> {
        Some(self)
    }

    fn to_list(&self) -> List {
        self.clone()
    }

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
}
