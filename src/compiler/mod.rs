//! Tokay compiler, parsing a program source into a VM program

pub(crate) mod ast;
mod compiler;
mod iml;
mod parser;
mod prelude;
mod scope;

use iml::*;
use parser::*;
use scope::*;

pub(crate) use ast::{RESERVED_KEYWORDS, RESERVED_TOKENS};
pub use compiler::Compiler;
