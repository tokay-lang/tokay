//! Tokay intermediate code representation
pub use crate::vm::*;

mod alt;
mod expect;
mod if_;
mod loop_;
mod not;
mod op;
mod parselet;
mod peek;
mod repeat;
mod sequence;
mod value;

pub use alt::*;
pub use expect::*;
pub use if_::*;
pub use loop_::*;
pub use not::*;
pub use op::*;
pub use parselet::*;
pub use peek::*;
pub use repeat::*;
pub use sequence::*;
pub(super) use value::*;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Consumable {
    pub leftrec: bool,  // Flag if consumable is left-recursive
    pub nullable: bool, // Flag if consumable is nullable
}

pub trait Compileable: std::fmt::Debug + std::fmt::Display
/* todo: (in crate::compiler) trait Compileable: std::fmt::Debug + std::fmt::Display */
{
    /** Resolve any unresolved Usages. */
    fn resolve(&mut self, usages: &mut Vec<Vec<ImlOp>>);

    /** Finalize program regarding grammar view flags;
    This function is called from top of each parselet to detect
    both left-recursive and nullable behaviors. */
    fn finalize(
        &mut self,
        values: &Vec<ImlValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<Consumable>;

    /** Turn intermediate structure into Tokay VM code. */
    fn compile(&self, parselet: &ImlParselet) -> Vec<Op>;

    /** Convert parser object into boxed dyn Parser Op */
    fn into_op(self) -> ImlOp
    where
        Self: Sized + 'static,
    {
        ImlOp::Compileable(Box::new(self))
    }
}
