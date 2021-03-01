use crate::reader::Reader;
use crate::value::RefValue;

mod op;
mod op_block;
mod op_chars;
mod op_expect;
mod op_match;
mod op_repeat;
mod op_sequence;
mod parselet;
mod program;
mod vm;

pub use op::*;
pub use op_block::*;
pub use op_chars::*;
pub use op_expect::*;
pub use op_match::*;
pub use op_repeat::*;
pub use op_sequence::*;
pub use parselet::*;
pub use program::*;
pub use vm::*;

pub trait Scanable: std::fmt::Debug + std::fmt::Display {
    fn scan(&self, reader: &mut Reader) -> Result<Accept, Reject>;

    /** Convert scanable into boxed dyn Scanable Op */
    fn into_op(self) -> Op
    where
        Self: Sized + 'static,
    {
        Op::Scanable(Box::new(self))
    }
}

pub trait Runable: std::fmt::Debug + std::fmt::Display {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject>;

    /** Finalize resolved usages and implement grammar view flags;
    This function is called from top of each parselet to detect
    both left-recursive and nullable (=no input consuming) structures. */
    fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        usages: &mut Vec<Vec<Op>>,
        leftrec: &mut bool,
        nullable: &mut bool,
    );

    /** Convert parser object into boxed dyn Parser Op */
    fn into_op(self) -> Op
    where
        Self: Sized + 'static,
    {
        Op::Runable(Box::new(self))
    }
}
