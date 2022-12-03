//! Dictionary object
use super::{BoxedObject, List, Object, RefValue};
use crate::value;
use crate::Error;
use indexmap::IndexMap;
use tokay_macros::tokay_method;
extern crate self as tokay;
use std::cmp::Ordering;

// Alias for the inner dict
type InnerDict = IndexMap<String, RefValue>;

// Dict object type
#[derive(Debug, Clone)]
pub struct Dict {
    dict: InnerDict,
}

impl Object for Dict {
    fn severity(&self) -> u8 {
        20
    }

    fn name(&self) -> &'static str {
        "dict"
    }

    fn repr(&self) -> String {
        if self.is_empty() {
            return "dict()".to_string();
        }

        let mut ret = "(".to_string();

        for (key, value) in self.iter() {
            if ret.len() > 1 {
                ret.push_str(", ");
            }

            // todo: Put this into a utility function...
            if !key.chars().all(|ch| ch.is_alphabetic() || ch == '_') {
                ret.push('"');
                for ch in key.chars() {
                    match ch {
                        '\\' => ret.push_str("\\\\"),
                        '\n' => ret.push_str("\\n"),
                        '\r' => ret.push_str("\\r"),
                        '\t' => ret.push_str("\\t"),
                        '"' => ret.push_str("\\\""),
                        _ => ret.push(ch),
                    }
                }
                ret.push('"');
            } else {
                ret.push_str(key);
            }

            ret.push_str(" => ");
            ret.push_str(&value.borrow().repr());
        }

        ret.push(')');
        ret
    }

    fn is_true(&self) -> bool {
        self.len() > 0
    }

    fn is_mutable(&self) -> bool {
        true
    }
}

impl Dict {
    pub fn new() -> Self {
        Self {
            dict: InnerDict::new(),
        }
    }

    tokay_method!("dict : @", Ok(RefValue::from(Dict::new())));

    tokay_method!("dict_len : @dict", {
        let dict = dict.borrow();

        if let Some(dict) = dict.object::<Dict>() {
            Ok(RefValue::from(dict.len()))
        } else {
            Err(Error::from(format!(
                "{} only accepts '{}' as parameter, not '{}'",
                __function,
                "dict",
                dict.name()
            )))
        }
    });

    tokay_method!("dict_keys : @dict", {
        let dict = dict.borrow();

        if let Some(dict) = dict.object::<Dict>() {
            let mut list = List::with_capacity(dict.len());

            for key in dict.keys() {
                list.push(RefValue::from(key.to_owned()));
            }

            Ok(RefValue::from(list))
        } else {
            Err(Error::from(format!(
                "{} only accepts '{}' as parameter, not '{}'",
                __function,
                "dict",
                dict.name()
            )))
        }
    });

    tokay_method!("dict_values : @dict", {
        let dict = dict.borrow();

        if let Some(dict) = dict.object::<Dict>() {
            let mut list = List::with_capacity(dict.len());

            for value in dict.values() {
                list.push(value.clone());
            }

            Ok(RefValue::from(list))
        } else {
            Err(Error::from(format!(
                "{} only accepts '{}' as parameter, not '{}'",
                __function,
                "dict",
                dict.name()
            )))
        }
    });

    tokay_method!("dict_items : @dict", {
        let dict = dict.borrow();

        if let Some(dict) = dict.object::<Dict>() {
            let mut list = List::with_capacity(dict.len());

            for (key, value) in dict.iter() {
                // fixme: wanted to shortcut this all with
                //  list.push(value!([key.to_string(), value.clone()]));
                // but doesn't compile.
                let mut item = List::with_capacity(2);

                item.push(RefValue::from(key.to_string()));
                item.push(value.clone());

                list.push(RefValue::from(item));
            }

            Ok(RefValue::from(list))
        } else {
            Err(Error::from(format!(
                "{} only accepts '{}' as parameter, not '{}'",
                __function,
                "dict",
                dict.name()
            )))
        }
    });

    tokay_method!("dict_get_item : @dict, item, default=void", {
        // todo: alias dict_get
        let dict = dict.borrow();
        let item = item.to_string();

        if let Some(dict) = dict.object::<Dict>() {
            if let Some(item) = dict.get(&item) {
                Ok(item.clone())
            } else {
                Ok(default)
            }
        } else {
            Err(Error::from(format!(
                "{} only accepts '{}' as parameter, not '{}'",
                __function,
                "dict",
                dict.name()
            )))
        }
    });

    tokay_method!("dict_set_item : @dict, item, value=void", {
        let mut dict = dict.borrow_mut();
        let item = item.to_string();

        if let Some(dict) = dict.object_mut::<Dict>() {
            if value.is_void() {
                dict.remove(&item);
                Ok(value![void])
            } else {
                dict.insert(item, value.clone());
                Ok(value)
            }
        } else {
            Err(Error::from(format!(
                "{} only accepts '{}' as parameter, not '{}'",
                __function,
                "dict",
                dict.name()
            )))
        }
    });

    tokay_method!("dict_merge : @dict, other", {
        {
            let dict = &mut *dict.borrow_mut();

            if let Ok(other) = other.try_borrow() {
                if let Some(dict) = dict.object_mut::<Dict>() {
                    if let Some(other) = other.object::<Dict>() {
                        for (k, v) in other.iter() {
                            dict.insert(k.clone(), v.clone());
                        }
                    } else {
                        return Err(Error::from(format!(
                            "{} only accepts '{}' as second parameter, not '{}'",
                            __function,
                            dict.name(),
                            other.name()
                        )));
                    }
                } else {
                    return Err(Error::from(format!(
                        "{} only accepts '{}' as first parameter, not '{}'",
                        __function,
                        "dict",
                        dict.name()
                    )));
                }
            }
        }

        Ok(dict)
    });

    tokay_method!("dict_push : @dict, key, value", {
        let dict = &mut *dict.borrow_mut();

        if let Some(dict) = dict.object_mut::<Dict>() {
            let key = key.to_string();

            Ok(if let Some(old) = dict.insert(key, value) {
                old
            } else {
                value!(void)
            })
        } else {
            Err(Error::from(format!(
                "{} only accepts '{}' as parameter, not '{}'",
                __function,
                "dict",
                dict.name()
            )))
        }
    });

    tokay_method!("dict_pop : @dict, key=void, default=void", {
        let dict = &mut *dict.borrow_mut();

        if let Some(dict) = dict.object_mut::<Dict>() {
            if key.is_void() {
                return Ok(if let Some(last) = dict.pop() {
                    last.1
                } else {
                    default
                });
            }

            let key = key.to_string();

            Ok(if let Some(value) = dict.remove(&key) {
                value
            } else {
                default
            })
        } else {
            Err(Error::from(format!(
                "{} only accepts '{}' as parameter, not '{}'",
                __function,
                "dict",
                dict.name()
            )))
        }
    });

    /*
    fn get_index(&self, index: &Value) -> Result<RefValue, String> {
        let index = index.to_string();
        if let Some(value) = self.get(&index) {
            Ok(value.clone())
        } else {
            Err(format!("Key '{}' not found", index))
        }
    }

    fn set_index(&mut self, index: &Value, value: RefValue) -> Result<(), String> {
        let index = index.to_string();
        self.insert(index, value);
        Ok(())
    }
    */
}

// Implement PartialOrd and PartialEq on our own,
// until https://github.com/bluss/indexmap/issues/153
// may become resolved.
impl PartialOrd for Dict {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.len() < other.len() {
            Some(Ordering::Less)
        } else if self.len() > other.len() {
            Some(Ordering::Greater)
        } else {
            for (a, b) in self.iter().zip(other.iter()) {
                if a.0 < b.0 || a.1 < b.1 {
                    return Some(Ordering::Less);
                } else if a.0 > b.0 || a.1 > b.1 {
                    return Some(Ordering::Greater);
                }
            }

            Some(Ordering::Equal)
        }
    }
}

impl PartialEq for Dict {
    fn eq(&self, other: &Self) -> bool {
        if self.id() == other.id() {
            return true;
        }

        if let Some(Ordering::Equal) = self.partial_cmp(other) {
            true
        } else {
            false
        }
    }
}

impl std::ops::Deref for Dict {
    type Target = InnerDict;

    fn deref(&self) -> &Self::Target {
        &self.dict
    }
}

impl std::ops::DerefMut for Dict {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.dict
    }
}

impl From<Dict> for RefValue {
    fn from(value: Dict) -> Self {
        RefValue::from(Box::new(value) as BoxedObject)
    }
}
