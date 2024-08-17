//! Lexical tokens for interoperable Whitespace assembly.

pub mod comment;
pub mod integer;
pub mod label;
pub(crate) mod mnemonics;
pub mod spaces;
pub mod string;
mod token;
pub mod words;

pub use token::*;
