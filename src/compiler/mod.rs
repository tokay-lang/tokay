//! Tokay compiler, parsing a program source into a VM program

pub(crate) mod ast;
mod compiler;
mod iml;
mod linker;
mod parser;

use compiler::*;
use iml::*;
use linker::*;
use parser::*;

pub use compiler::Compiler;
