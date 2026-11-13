//! Go `checker/pseudotypenodebuilder.go`: maps pseudochecker pseudo types
//! into type nodes, and compares pseudo types with checker types.
//!
//! Methods use the Relater pattern: `impl Checker` methods take `&mut self`
//! and `b: &Rc<RefCell<NodeBuilderImpl>>`. No `RefCell` borrow of `b` is held
//! across a Checker call.

use crate::prelude::*;
use crate::printer::EmitFlags;
use crate::pseudochecker::{
    self, PseudoObjectElementKind, PseudoParameter, PseudoType, PseudoTypeKind,
};

use super::nodebuilder_impl_p3::{};

// Go: checker/pseudotypenodebuilder.go:632 isStructuralPseudoType

