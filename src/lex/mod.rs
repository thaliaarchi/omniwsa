//! Generic token scanning.

pub mod byte_trie;
mod scan;
mod token_stream;

pub use scan::*;
pub(crate) use token_stream::*;
