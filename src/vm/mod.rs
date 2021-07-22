//! Tokay virtual machine

use crate::value::RefValue;

mod op;
mod op_block;
mod op_expect;
mod op_if;
mod op_loop;
mod op_not;
mod op_peek;
mod op_repeat;
mod op_sequence;
mod parselet;
mod program;
mod vm;

pub use op::*;
pub use op_block::*;
pub use op_expect::*;
pub use op_if::*;
pub use op_loop::*;
pub use op_not::*;
pub use op_peek::*;
pub use op_repeat::*;
pub use op_sequence::*;
pub use parselet::*;
pub use program::*;
pub use vm::*;

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
