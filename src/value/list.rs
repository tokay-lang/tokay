//! List object
use super::{BoxedObject, Object, RefValue};
use macros::tokay_method;

/// Alias for the inner list definition
type InnerList = Vec<RefValue>;

/// List object type
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct List {
    list: InnerList,
}

impl Object for List {
    fn severity(&self) -> u8 {
        30
    }

    fn name(&self) -> &'static str {
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
}

impl List {
    pub fn new() -> Self {
        Self {
            list: InnerList::new(),
        }
    }

    tokay_method!("list_new(*args)", {
        let list = if args.len() == 1 {
            List::from(args[0].clone())
        } else {
            List { list: args }
        };

        Ok(RefValue::from(list))
    });

    tokay_method!("list_concat(list, append)", {
        // In case list is not a list, make it a list.
        if !list.is("list") {
            list = Self::list_new(vec![list.clone()], None)?;
        }

        {
            let mut list = list.borrow_mut();
            let list = list.object_mut::<List>().unwrap();

            // When append is a list, append all items to list
            if let Some(append) = append.borrow().object::<List>() {
                list.reserve(append.len());

                for item in append.iter() {
                    list.push(item.clone());
                }
            // Otherwise, just push append to the list.
            } else {
                list.push(append.clone());
            }
        }

        Ok(list)
    });

    tokay_method!("list_add(list, append)", {
        if !list.is("list") {
            list = Self::list_new(vec![list.clone()], None)?;
        }

        let list = list.borrow();
        let mut list = list.object::<List>().unwrap().clone();

        // When append is a list, append all items to list
        if let Some(append) = append.borrow().object::<List>() {
            list.reserve(append.len());

            for item in append.iter() {
                list.push(item.clone());
            }
        // Otherwise, just push append to the list.
        } else {
            list.push(append.clone());
        }

        Ok(RefValue::from(list))
    });

    tokay_method!("list_push(list, item)", {
        // Push the item to the list
        if list.is("list") {
            list.borrow_mut().object_mut::<List>().unwrap().push(item);
        }
        // If list is not a list, turn it into a list and push list as first element
        else {
            list = Self::list_new(vec![list.clone()], None)?;
        }

        Ok(list)
    });
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

impl From<RefValue> for List {
    fn from(refvalue: RefValue) -> Self {
        if let Some(list) = refvalue.borrow().object::<List>() {
            (*list).clone()
        } else {
            Self {
                list: vec![refvalue.clone()],
            }
        }
    }
}

impl From<&RefValue> for List {
    fn from(refvalue: &RefValue) -> Self {
        List::from(refvalue.clone())
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
        RefValue::from(Box::new(value) as BoxedObject)
    }
}
