//! Tokay compiler, parsing a program source into a VM program

pub(crate) mod ast;
mod compiler;
mod iml;
mod linker;
mod macros;
mod parser;

use compiler::*;
use iml::*;
use linker::*;
use parser::*;

pub(crate) use ast::identifier_is_valid;
pub use compiler::Compiler;
