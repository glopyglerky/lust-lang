//! `crate::prelude` of `goport_util`: the names of the `ts_goport` prelude
//! that util files use. Util files import it as `use crate::prelude::*;`.

pub use crate::core::*;
pub use crate::diag;
pub use crate::scanner_util::*;
pub use crate::unported;
pub use crate::indexmap::{IndexMap, IndexSet};
pub use crate::rustc_hash::{FxHashMap, FxHashSet};
pub use std::cell::RefCell;
pub use std::rc::Rc;
