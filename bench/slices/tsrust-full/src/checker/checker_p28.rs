//! Port of typescript-go `checker/checker.go` lines 24957-25911 (unit checker-28):
//! type constructors, literal types, widening helpers, `mapType` and union
//! type construction and reduction.

use crate::checker::relater_p3::TemplateLiteralEnds;
use crate::jsnum::{Number, PseudoBigInt};
use crate::prelude::*;
use crate::rustc_hash::FxBuildHasher;
use std::borrow::Cow;
use std::hash::BuildHasher;

