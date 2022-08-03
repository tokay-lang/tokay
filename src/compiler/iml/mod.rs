//! Tokay intermediate code representation
pub use crate::vm::*;

mod op;
mod parselet;
mod usage;
mod value;

use super::Linker;
pub(crate) use op::*;
pub(crate) use parselet::*;
pub(crate) use usage::*;
pub(super) use value::*;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Consumable {
    pub leftrec: bool,  // Flag if consumable is left-recursive
    pub nullable: bool, // Flag if consumable is nullable
}
