//! Lexical tokens for interoperable Whitespace assembly.

pub mod integer;
pub(crate) mod mnemonics;
pub mod string;
mod token;

pub use token::*;
