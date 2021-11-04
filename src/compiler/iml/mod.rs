//! Tokay intermediate code representation
use crate::value::RefValue;
pub use crate::vm::*;

mod iml_alt;
mod iml_expect;
mod iml_if;
mod iml_loop;
mod iml_not;
mod iml_peek;
mod iml_repeat;
mod iml_sequence;
mod op;
mod parselet;

pub use iml_alt::*;
pub use iml_expect::*;
pub use iml_if::*;
pub use iml_loop::*;
pub use iml_not::*;
pub use iml_peek::*;
pub use iml_repeat::*;
pub use iml_sequence::*;
pub use op::*;
pub use parselet::*;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub /* todo: (in crate::compiler) */ struct Consumable {
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
        statics: &Vec<RefValue>,
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
