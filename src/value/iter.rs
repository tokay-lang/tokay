//! An iterator, probably running on a given object
use super::{Object, RefValue, Value};
use crate::value;
use crate::Error;
use num_bigint::BigInt;
use std::cell::RefCell;
use std::rc::Rc;
use tokay_macros::{tokay_function, tokay_method};
extern crate self as tokay;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct MethodOpIter {
    pub object: RefValue,
    pub method: &'static str,
    pub index: Option<RefValue>,
    pub inc_op: &'static str,
    pub rindex: Option<RefValue>,
    pub dec_op: &'static str,
}

impl Iterator for MethodOpIter {
    type Item = RefValue;

    fn next(&mut self) -> Option<Self::Item> {
        if let (Some(index), Some(rindex)) = (&self.index, &self.rindex) {
            if index <= rindex {
                match self.object.call_method(self.method, vec![index.clone()]) {
                    Ok(Some(next)) => {
                        // When next is not void, increment index and return next
                        if !next.is_void() {
                            self.index = Some(index.clone().unary_op(self.inc_op).unwrap());
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
            }

            self.index = None; // Invalidate this iterator
        }

        None
    }
}

impl DoubleEndedIterator for MethodOpIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let (Some(rindex), Some(index)) = (&self.rindex, &self.index) {
            let rindex = rindex.clone().unary_op(self.dec_op).unwrap();

            if &rindex >= index {
                match self.object.call_method(self.method, vec![rindex.clone()]) {
                    Ok(Some(next)) => {
                        // When next is not void, increment index and return next
                        if !next.is_void() {
                            self.rindex = Some(rindex);
                            return Some(next);
                        }
                    }
                    _ => {
                        // Special case: Return the object itself once as its own iter
                        if !rindex.is_true() {
                            return Some(self.object.clone());
                        }
                    }
                }
            }

            self.rindex = None; // Invalidate this iterator
        }

        None
    }
}

#[derive(Clone)]
pub enum Iter {
    MethodOp(MethodOpIter),
    Builtin {
        iter: Rc<RefCell<dyn DoubleEndedIterator<Item = RefValue>>>,
        repr: String,
    },
}

impl Iter {
    /// Creates a new iterator on object, with default "get_item"-method and "iinc"-operation.
    pub fn new(object: RefValue) -> Self {
        Iter::new_method_op(
            object.clone(),
            "get_item",
            None,
            "iinc",
            object.call_method("len", Vec::new()).unwrap(),
            "idec",
        )
    }

    /// Creates a new iterator on object, using item retrieval method and op operation.
    /// index can be set to an optional start value; If None, the iterator will be initialized with Some(0).
    pub fn new_method_op(
        object: RefValue,
        method: &'static str,
        index: Option<RefValue>,
        inc_op: &'static str,
        rindex: Option<RefValue>,
        dec_op: &'static str,
    ) -> Self {
        Self::MethodOp(MethodOpIter {
            object: object.clone(),
            method,
            inc_op,
            index: index.or_else(|| Some(value!(0))),
            dec_op,
            rindex,
        })
    }

    tokay_method!("iter : @value", {
        if value.is("iter") || value.is_void() {
            Ok(value)
        }
        // Check for an available iter() method on the provided value first
        else if let Ok(Some(iter)) = value.call_method("iter", Vec::new()) {
            Ok(iter)
        }
        // Default fallback to Iter on the object
        else {
            Ok(RefValue::from(Iter::new(value)))
        }
    });

    tokay_method!("iter_next : @iter", {
        let mut iter = iter.borrow_mut();

        if let Some(iter) = iter.object_mut::<Iter>() {
            Ok(RefValue::from(iter.next().unwrap_or_else(|| value!(void))))
        } else {
            Err(Error::from(format!(
                "{} only accepts '{}' as parameter, not '{}'",
                __function,
                "iter",
                iter.name()
            )))
        }
    });

    tokay_method!("iter_prev : @iter", {
        let mut iter = iter.borrow_mut();

        if let Some(iter) = iter.object_mut::<Iter>() {
            Ok(RefValue::from(
                iter.next_back().unwrap_or_else(|| value!(void)),
            ))
        } else {
            Err(Error::from(format!(
                "{} only accepts '{}' as parameter, not '{}'",
                __function,
                "iter",
                iter.name()
            )))
        }
    });

    tokay_method!("iter_len : @iter", {
        let mut iter = iter.borrow_mut();

        Ok(RefValue::from(
            if let Some(iter) = iter.object_mut::<Iter>() {
                iter.count()
            } else {
                1
            },
        ))
    });
}

impl Iterator for Iter {
    type Item = RefValue;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::MethodOp(iter) => iter.next(),
            Self::Builtin { iter, .. } => {
                let mut iter = iter.borrow_mut();
                iter.next()
            }
        }
    }
}

impl DoubleEndedIterator for Iter {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::MethodOp(iter) => iter.next_back(),
            Self::Builtin { iter, .. } => {
                let mut iter = iter.borrow_mut();
                iter.next_back()
            }
        }
    }
}

impl Object for Iter {
    fn name(&self) -> &'static str {
        "iter"
    }

    fn repr(&self) -> String {
        match self {
            Self::MethodOp(MethodOpIter { object, .. }) => {
                let mut repr = object.repr();
                if repr.starts_with("<") && repr.ends_with(">") {
                    repr = repr[1..repr.len() - 1].to_string();
                }

                format!(
                    "<{} {} of {} object at {:#x}>",
                    self.name(),
                    repr,
                    object.name(),
                    object.id()
                )
            }
            Self::Builtin { repr, .. } => repr.clone(),
        }
    }
}

impl std::fmt::Debug for Iter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PartialEq for Iter {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl PartialOrd for Iter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id().partial_cmp(&other.id())
    }
}

impl From<Iter> for RefValue {
    fn from(iter: Iter) -> Self {
        Value::Object(Box::new(iter)).into()
    }
}

struct RangeIterator {
    next: Option<BigInt>,
    stop: Option<BigInt>,
    step: BigInt,
}

impl Iterator for RangeIterator {
    type Item = RefValue;

    fn next(&mut self) -> Option<Self::Item> {
        if let (Some(next), Some(stop)) = (self.next.as_mut(), &self.stop) {
            if *next < *stop {
                let ret = next.clone();
                *next += &self.step;
                return Some(RefValue::from(ret));
            }

            self.next = None;
            self.stop = None;
        }

        None
    }
}

impl DoubleEndedIterator for RangeIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let (Some(next), Some(stop)) = (&self.next, self.stop.as_mut()) {
            if *stop > *next {
                *stop -= &self.step;
                return Some(RefValue::from(stop.clone()));
            }

            self.next = None;
            self.stop = None;
        }

        None
    }
}

tokay_function!("range : @start, stop=void, step=1", {
    let start = if stop.is_void() {
        stop = start;
        BigInt::from(0)
    } else {
        start.to_bigint()?
    };

    let stop = stop.to_bigint()?;
    let step = step.to_bigint()?;

    RefValue::from(Iter::Builtin {
        repr: format!("range({}, {}, {})", start, stop, step),
        iter: Rc::new(RefCell::new(RangeIterator {
            next: Some(start),
            stop: Some(stop),
            step,
        })),
    })
    .into()
});
