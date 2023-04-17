//! Tokay intermediate code representation
pub use crate::vm::*;

mod imlop;
mod imlparselet;
mod imlprogram;
mod imlvalue;

pub(in crate::compiler) use imlop::*;
pub(in crate::compiler) use imlparselet::*;
pub(in crate::compiler) use imlprogram::*;
pub(in crate::compiler) use imlvalue::*;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub(in crate::compiler) struct Consumable {
    pub leftrec: bool,  // Flag if consumable is left-recursive
    pub nullable: bool, // Flag if consumable is nullable
}
