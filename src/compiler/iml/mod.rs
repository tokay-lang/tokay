//! Tokay intermediate code representation
use crate::vm::*;
use crate::value::RefValue;

mod iml;
mod iml_block;
mod iml_expect;
mod iml_if;
mod iml_loop;
mod iml_not;
mod iml_peek;
mod iml_repeat;
mod iml_sequence;
mod parselet;

pub use iml::*;
pub use iml_block::*;
pub use iml_expect::*;
pub use iml_if::*;
pub use iml_loop::*;
pub use iml_not::*;
pub use iml_peek::*;
pub use iml_repeat::*;
pub use iml_sequence::*;
pub use parselet::*;

pub trait Runable: std::fmt::Debug + std::fmt::Display {
    // Run that runable...
    fn run(&self, context: &mut Context) -> Result<Accept, Reject>;

    /** Resolve any unresolved Usages. */
    fn resolve(&mut self, usages: &mut Vec<Vec<Op>>);

    /** Finalize program regarding grammar view flags;
    This function is called from top of each parselet to detect
    both left-recursive and nullable behaviors. */
    fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<(bool, bool)>;

    /** Convert parser object into boxed dyn Parser Op */
    fn into_op(self) -> Op
    where
        Self: Sized + 'static,
    {
        Op::Runable(Box::new(self))
    }
}
