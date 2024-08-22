//! Parsing for Whitespace assembly dialects.

mod burghard;
mod palaiologos;
#[expect(dead_code)]
mod wconrad;

pub use burghard::Burghard;
pub use palaiologos::Palaiologos;
pub use wconrad::WConrad;
