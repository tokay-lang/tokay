// Tokay
// Copyright © 2021 by Jan Max Meyer, Phorward Software Technologies.
// Licensed under the MIT license. See LICENSE for more information.

/*! Tokay

    An imperative, procedural programming language dedicated to parsing and other text-processing tasks.

    Visit [https://tokay.dev](https://tokay.dev) for more information.
*/

pub mod builtin;
pub mod compiler;
pub mod error;
pub mod reader;
pub mod repl;
#[cfg(test)]
pub mod test;
pub mod utils;
pub mod value;
pub mod vm;
