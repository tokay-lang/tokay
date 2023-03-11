/*!
An iterator for enumerating an index on every item.
*/
use crate::value::{Iter, Object, RefValue, RefValueIter};
use crate::{Context, Error};
use num_bigint::BigInt;
use tokay_macros::tokay_method;
extern crate self as tokay;

#[derive(Clone)]
pub struct EnumIter {
    pub iter: RefValue, // Iterator
    count: BigInt,
}

impl EnumIter {
    /// Creates a enumerated iterator on an iterator.
    pub fn new(iter: RefValue) -> Iter {
        assert!(iter.is("iter"));
        Iter::new(Box::new(Self {
            iter,
            count: BigInt::from(0),
        }))
    }

    tokay_method!("iter_enum : @iter", {
        if !iter.is("iter") {
            return Err(Error::from("'iter' must be of type iter"));
        }

        Ok(RefValue::from(Self::new(iter)))
    });
}

impl RefValueIter for EnumIter {
    fn next(&mut self, mut context: Option<&mut Context>) -> Option<RefValue> {
        if let Some(next) = {
            let mut iter = self.iter.borrow_mut();
            let iter = iter.object_mut::<Iter>().expect("Iter object expected");
            iter.iter.next(context.as_deref_mut())
        } {
            let ret = RefValue::from(vec![RefValue::from(self.count.clone()), next]);
            self.count += 1;
            return Some(ret);
        }

        None
    }

    fn repr(&self) -> String {
        format!("<EnumIter on {} at {}>", self.iter.repr(), self.count,)
    }

    fn rev(&mut self) -> Result<(), Error> {
        Iter::iter_rev(vec![self.iter.clone()], None)?;
        Ok(())
    }
}
