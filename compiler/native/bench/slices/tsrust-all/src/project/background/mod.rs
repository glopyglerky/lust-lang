//! Go package `internal/project/background`.

pub mod queue;

pub use queue::*;

/// Glob import for background files: `use crate::project::background::prelude::*;`.
pub mod prelude {
    pub use super::queue::*;
    pub use crate::gostd::{self, GoError};
    pub use crate::prelude::*;
}
