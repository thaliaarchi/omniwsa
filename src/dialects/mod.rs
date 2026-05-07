//! Parsing for Whitespace assembly dialects.

mod burghard;
mod censoredusername;
mod dialect;
mod iczelia;
mod voliva;
mod wconrad;
mod wsf;

pub use burghard::Burghard;
pub use censoredusername::CensoredUsername;
pub use dialect::*;
pub use iczelia::Iczelia;
pub use voliva::Voliva;
pub use wconrad::WConrad;
pub use wsf::Wsf;
