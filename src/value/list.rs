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

        ret.push(')');

        if self.len() <= 1 {
            ret = format!("list{}", ret);
        }

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

    tokay_method!("list(*args)", {
        let list = if args.len() == 1 {
            List::from(args[0].clone())
        } else {
            List { list: args }
        };

        Ok(RefValue::from(list))
    });

    tokay_method!("list_iadd(list, append)", {
        // In case list is not a list, make it a list.
        if !list.is("list") {
            let item = RefValue::from(list.borrow().clone());
            list = Self::list(vec![item], None)?;
        }

        // Extend in-place when possible.
        if let (Ok(mut inner), Ok(to_append)) = (list.try_borrow_mut(), append.try_borrow()) {
            let inner = inner.object_mut::<List>().unwrap();

            // When append is a list, append all items to list
            if let Some(append) = to_append.object::<List>() {
                inner.reserve(append.len());

                for item in append.iter() {
                    inner.push(item.clone());
                }
            // Otherwise, just push append to the list.
            } else {
                inner.push(append.clone());
            }

            return Ok(list.clone());
        }

        // Otherwise, perform ordinary add first, then re-assign to list
        let new = Self::list_add(vec![list.clone(), append.clone()], None)?;
        *list.borrow_mut() = new.into();

        Ok(list)
    });

    tokay_method!("list_add(list, append)", {
        // In case list is not a list, make it a list.
        if !list.is("list") {
            list = Self::list(vec![list], None)?;
        }

        let mut list = list.borrow().object::<List>().unwrap().clone();

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

    tokay_method!("list_push(list, append)", {
        if !list.is("list") {
            list = Self::list(vec![list], None)?;
        }

        {
            let mut list = list.borrow_mut();
            let list = list.object_mut::<List>().unwrap();
            list.push(append.clone());
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

#[test]
fn test_list_new() {
    assert_eq!(
        crate::utils::compile_and_run("list(true) list((1,2,3)) list(\"Tokay\")", ""),
        Ok(Some(crate::value!([[true], [1, 2, 3], ["Tokay"]])))
    )
}

#[test]
fn test_list_iadd() {
    assert_eq!(
        crate::utils::compile_and_run("l = list(1); l += 2; l += (3, 4); l", ""),
        Ok(Some(crate::value!([1, 2, 3, 4])))
    )
}

#[test]
fn test_list_add() {
    assert_eq!(
        crate::utils::compile_and_run("l = list(1); l + (2, 3) l", ""),
        Ok(Some(crate::value!([[1, 2, 3], [1]])))
    )
}

#[test]
fn test_list_push() {
    assert_eq!(
        crate::utils::compile_and_run("l = list(1); l.push(2); l.push((3, 4)); l", ""),
        Ok(Some(crate::value!([1, 2, [3, 4]])))
    )
}

#[test]
fn test_list_repr() {
    /*
    Currently under consideration, see https://github.com/tokay-lang/tokay/issues/45
    assert_eq!(
        crate::utils::compile_and_run("repr((1, ))", ""),
        Ok(Some(crate::value!("(1, )")))
    );
    */

    assert_eq!(
        crate::utils::compile_and_run("l = list(); l += 1; repr(l)", ""),
        Ok(Some(crate::value!("list(1)")))
    );

    assert_eq!(
        crate::utils::compile_and_run("repr((1, 2, 3, 4))", ""),
        Ok(Some(crate::value!("(1, 2, 3, 4)")))
    )
}
