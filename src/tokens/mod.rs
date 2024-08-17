//! Lexical tokens for interoperable Whitespace assembly.

pub mod integer;
pub(crate) mod mnemonics;
pub mod spaces;
pub mod string;
mod token;
pub mod words;

pub use token::*;
