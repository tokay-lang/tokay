//! Tokay compiler, parsing a program source into a VM program

pub(crate) mod ast;
mod compiler;
mod iml;
mod macros;
mod parser;
#[cfg(test)]
mod test;
mod usage;

pub use compiler::*;
use iml::*;
pub use macros::*;
pub use parser::*;
pub use usage::*;
