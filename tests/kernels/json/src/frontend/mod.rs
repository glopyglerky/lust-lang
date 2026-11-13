//! Kernel mirror of the ts-rust `frontend` module paths, so the copied
//! files keep their `crate::frontend::…` paths unchanged.

pub mod json;
pub mod json_ext;
pub mod json_indexmap;

/// Kernel mirror of `crate::frontend::prelude` (in ts-rust, the
/// `goport_util` util prelude). Only the names the kernel files use.
pub mod prelude {
    pub use crate::frontend::json::*;
    pub use crate::prelude::*;
}
