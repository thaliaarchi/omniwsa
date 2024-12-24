#![doc = include_str!("../README.md")]
#![allow(clippy::manual_is_ascii_check, clippy::module_inception)]

pub mod codegen;
pub mod dialects;
pub mod lex;
pub mod syntax;
#[cfg(test)]
mod tests;
pub mod tokens;
pub mod transform;
pub mod ws;
