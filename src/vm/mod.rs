use crate::value::RefValue;

mod op;
mod op_block;
mod op_expect;
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
pub use op_not::*;
pub use op_peek::*;
pub use op_repeat::*;
pub use op_sequence::*;
pub use parselet::*;
pub use program::*;
pub use vm::*;

pub trait Runable: std::fmt::Debug + std::fmt::Display {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject>;

    /** Finalize resolved usages and implement grammar view flags;
    This function is called from top of each parselet to detect
    both left-recursive and nullable (=no input consuming) structures. */
    fn finalize(
        &mut self,
        usages: &mut Vec<Vec<Op>>,
        statics: &Vec<RefValue>,
        leftrec: Option<&mut bool>,
        nullable: &mut bool,
        consumes: &mut bool,
    );

    /** Convert parser object into boxed dyn Parser Op */
    fn into_op(self) -> Op
    where
        Self: Sized + 'static,
    {
        Op::Runable(Box::new(self))
    }
}