/*!
An iterator, running on a specific object using an index, yielding items that are
returned by a specific method call with the index. The index is calculated with a
given unary operation (by default, iinc and idec)
*/
use crate::value::{Iter, Object, RefValue, RefValueIter};
use crate::{Context, Error};
use tokay_macros::tokay_method;
extern crate self as tokay;

#[derive(Clone)]
pub struct MapIter {
    pub iter: RefValue, // Iterator
    pub map: RefValue,  // Mapping
}

impl MapIter {
    /// Creates a new iterator on an iterator with a mapping
    pub fn new(iter: RefValue, map: RefValue) -> Iter {
        assert!(iter.is("iter"));
        assert!(map.is_callable(false));

        Iter::new(Box::new(Self { iter, map }))
    }

    tokay_method!("iter_map : @iter, map", {
        if !iter.is("iter") {
            Err(Error::from("'iter' must be of type iter"))
        } else if !map.is_callable(false) {
            Err(Error::from("'map' must be a callable accepting arguments"))
        } else {
            Ok(RefValue::from(Iter::new(Box::new(Self { iter, map }))))
        }
    });
}

impl RefValueIter for MapIter {
    fn next(&mut self, mut context: Option<&mut Context>) -> Option<RefValue> {
        while let Some(next) = {
            let mut iter = self.iter.borrow_mut();
            let iter = iter.object_mut::<Iter>().expect("Iter object expected");
            iter.iter.next(context.as_deref_mut())
        } {
            let ret = self.map.call(context.as_deref_mut(), vec![next], None);

            let value = match ret {
                Ok(accept) => accept.into_refvalue(),
                Err(_) => tokay::value!(void),
            };

            if !value.is_void() {
                return Some(value);
            }
        }

        None
    }

    fn repr(&self) -> String {
        format!(
            "<MapIter on {} using {}>",
            self.iter.repr(),
            self.map.repr(),
        )
    }

    fn rev(&mut self) -> Result<(), Error> {
        Iter::iter_rev(vec![self.iter.clone()], None)?;
        Ok(())
    }
}
