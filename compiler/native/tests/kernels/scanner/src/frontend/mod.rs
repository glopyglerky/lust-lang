//! Kernel mirror of the ts-rust frontend module paths, so the copied
//! scanner files keep their crate paths unchanged.

pub mod scanner;

/// Kernel mirror of crate::frontend::prelude (in ts-rust, the full
/// frontend prelude). Only the names the scanner files use.
pub mod prelude {
    pub use crate::diagnostics::Message;
    pub use crate::frontend::scanner::*;
    pub use crate::prelude::*;
}
