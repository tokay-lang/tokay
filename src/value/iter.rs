//! An iterator, probably running on a given object
use super::{Object, RefValue, Value};
use crate::value;
use crate::Error;
use num::{One, Zero};
use num_bigint::BigInt;
use tokay_macros::{tokay_function, tokay_method};
use dyn_clone::DynClone;
extern crate self as tokay;

pub trait RefValueIter: DynClone {
    fn next(&mut self) -> Option<RefValue>;
    fn repr(&self) -> String;
    fn rev(&mut self) -> Result<(), Error> {
        Err(Error::from("This iterator cannot be reversed."))
    }
}

dyn_clone::clone_trait_object!(RefValueIter);

#[derive(Clone)]
pub struct MethodIter {
    pub object: RefValue,
    pub object_method: &'static str,
    pub index: Option<RefValue>,
    pub index_op: &'static str,
}

impl MethodIter {
    /// Creates a new iterator on object, with default "get_item"-method and "iinc"-operation.
    pub fn new(object: RefValue) -> Iter {
        Self::new_method_iter(object, "get_item", None, "iinc")
    }

    /// Creates a new iterator on object, using item retrieval method and op operation.
    /// index can be set to an optional start value; If None, the iterator will be initialized with Some(0).
    pub fn new_method_iter(
        object: RefValue,
        object_method: &'static str,
        index: Option<RefValue>,
        index_op: &'static str,
    ) -> Iter {
        Iter {
            iter: Box::new(Self {
                object: object.clone(),
                object_method,
                index: index.or_else(|| Some(value!(0))),
                index_op,
            }),
        }
    }
}

impl RefValueIter for MethodIter {
    fn next(&mut self) -> Option<RefValue> {
        if let Some(index) = &self.index {
            match self
                .object
                .call_method(self.object_method, vec![index.clone()])
            {
                Ok(Some(next)) => {
                    // When next is not void, increment index and return next
                    if !next.is_void() {
                        self.index = Some(index.clone().unary_op(self.index_op).unwrap());
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

    fn repr(&self) -> String {
        if let Some(index) = &self.index {
            format!(
                "<MethodIter on {}_{} at {}, using {}>",
                self.object.name(),
                self.object_method,
                index.repr(),
                self.index_op,
            )
        } else {
            format!(
                "<MethodIter on {}_{} finished, used {}>",
                self.object.name(),
                self.object_method,
                self.index_op,
            )
        }
    }

    fn rev(&mut self) -> Result<(), Error> {
        match self.index_op {
            "iinc" => {
                self.index_op = "idec";

                match self
                    .object
                    .call_method("len", Vec::new())
                    .unwrap_or_else(|_| Some(value!(1)))
                {
                    Some(len) => {
                        self.index = Some(len.unary_op(self.index_op)?);
                        Ok(())
                    }
                    None => Err(Error::from("This iterator cannot be reversed.")),
                }
            }
            "idec" => {
                self.index_op = "iinc";
                self.index = Some(value!(0));
                Ok(())
            }
            _ => Err(Error::from("This iterator cannot be reversed.")),
        }
    }
}

#[derive(Clone)]
struct RangeIter {
    next: Option<BigInt>,
    stop: BigInt,
    step: BigInt,
}

impl RefValueIter for RangeIter {
    fn next(&mut self) -> Option<RefValue> {
        if let Some(next) = self.next.as_mut() {
            if *next != self.stop {
                let ret = next.clone();
                *next += &self.step;
                return Some(RefValue::from(ret));
            }

            self.next = None;
        }

        None
    }

    fn repr(&self) -> String {
        if self.step.is_one() {
            format!(
                "range({}, {})",
                self.next.as_ref().unwrap_or(&self.stop),
                self.stop
            )
        } else {
            format!(
                "range({}, {}, {})",
                self.next.as_ref().unwrap_or(&self.stop),
                self.stop,
                self.step
            )
        }
    }

    fn rev(&mut self) -> Result<(), Error> {
        let next = self.next.as_ref().unwrap_or(&self.stop).clone();
        (self.next, self.stop) = (Some(self.stop.clone()), next);
        self.step = -self.step.clone();
        Ok(())
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

    if step.is_zero() {
        return Error::from(format!("{} argument 'step' may not be 0", __function)).into();
    }

    RefValue::from(Iter {
        iter: Box::new(RangeIter {
            next: if (step > BigInt::zero() && start > stop)
                || (step < BigInt::zero() && stop > start)
            {
                None
            } else {
                Some(start)
            },
            stop,
            step,
        }),
    })
    .into()
});

#[derive(Clone)]
pub struct Iter {
    iter: Box<dyn RefValueIter>
}

impl Iter {
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
            Ok(RefValue::from(MethodIter::new(value)))
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

    tokay_method!("iter_rev : @iter", {
        {
            let mut iter = iter.borrow_mut();

            if let Some(iter) = iter.object_mut::<Iter>() {
                iter.iter.rev()?;
            } else {
                return Err(Error::from(format!(
                    "{} only accepts '{}' as parameter, not '{}'",
                    __function,
                    "iter",
                    iter.name()
                )));
            }
        }

        Ok(iter)
    });
}

impl Iterator for Iter {
    type Item = RefValue;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl Object for Iter {
    fn name(&self) -> &'static str {
        "iter"
    }

    fn repr(&self) -> String {
        self.iter.repr()
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
