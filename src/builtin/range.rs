use crate::value::{Iter, Object, RefValue, RefValueIter};
use crate::{Context, Error};
use num::{One, Zero};
use num_bigint::BigInt;
use tokay_macros::tokay_function;
extern crate self as tokay;

#[derive(Clone)]
struct RangeIter {
    next: Option<BigInt>,
    stop: BigInt,
    step: BigInt,
}

impl RefValueIter for RangeIter {
    fn next(&mut self, _context: Option<&mut Context>) -> Option<RefValue> {
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
        self.step = -self.step.clone();
        let next = self.next.as_ref().unwrap_or(&self.stop).clone();
        (self.next, self.stop) = (Some(self.stop.clone() + &self.step), next + &self.step);
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

    RefValue::from(Iter::new(Box::new(RangeIter {
        next: if (step > BigInt::zero() && start > stop) || (step < BigInt::zero() && stop > start)
        {
            None
        } else {
            Some(start)
        },
        stop,
        step,
    })))
    .into()
});
