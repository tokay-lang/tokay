//! Tokay intermediate code representation
pub use crate::vm::*;

mod op;
mod parselet;
mod value;

use super::Linker;
pub(in crate::compiler) use op::*;
pub(in crate::compiler) use parselet::*;
pub(in crate::compiler) use value::*;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub(in crate::compiler) struct Consumable {
    pub leftrec: bool,  // Flag if consumable is left-recursive
    pub nullable: bool, // Flag if consumable is nullable
}
