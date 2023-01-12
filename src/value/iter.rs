//! An iterator, probably running on a given object
use super::{Object, RefValue};
use crate::value;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Iter {
    object: RefValue,
    index: Option<RefValue>,
}

impl Iter {
    fn new(object: RefValue) -> Self {
        Self {
            object,
            index: Some(value!(0)),
        }
    }
}

impl Iterator for Iter {
    type Item = RefValue;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.index.clone() {
            match self.object.call_method("get_item", vec![index.clone()]) {
                Ok(Some(next)) => {
                    // When next is not void, increment index and return next
                    if !next.is_void() {
                        self.index = Some(index.unary_op("iinc").unwrap());
                        return Some(next);
                    }
                }
                _ => {
                    // Special case: Return the object itself once as its own iter
                    if !index.is_true() {
                        self.index = None; // Invalidate this iterator
                        return Some(self.object.clone());
                    }
                }
            }

            self.index = None; // Invalidate this iterator
        }

        None
    }
}

/*
impl Object for Iter {
    fn name(&self) -> &'static str {
        "iter"
    }

    fn repr(&self) -> String {
        let mut repr = self.method.repr();
        if repr.starts_with("<") && repr.ends_with(">") {
            repr = repr[1..repr.len() - 1].to_string();
        }

        format!(
            "<{} {} of {} object at {:#x}>",
            self.name(),
            repr,
            self.object.name(),
            self.object.id()
        )
    }
}

impl From<Iter> for RefValue {
    fn from(iter: Iter) -> Self {
        Value::Object(Box::new(iter)).into()
    }
}
*/

#[test]
fn iter() {
    let list = value!([1, 2, 3, 99]);
    let iter = Iter::new(list);

    for (i, value) in iter.enumerate() {
        println!("{} => {:?}", i, value);
    }

    let dict = value!(["a" => 1, "b" => 2, "c" => 3, "d" => 99]);
    let iter = Iter::new(dict);

    for (i, value) in iter.enumerate() {
        println!("{} => {:?}", i, value);
    }
}
