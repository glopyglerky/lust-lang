//! Port of execute/incremental/referencemap.go.
//!
//! PORT: Go `collections.SyncMap` and `collections.Set` are `FxIndexMap`
//! and `FxIndexSet`. Go iterates these maps in random order; the port
//! iterates in insertion order so that runs are repeatable. Every Go caller
//! either sorts the result or does not depend on the order.
//!
//! PERF: the keys are long paths, so the maps and sets use the Fx hasher
//! instead of SipHash. The hasher does not change the order.

use crate::frontend::prelude::*;
use std::cell::OnceCell;
use std::sync::Arc;

// Go: incremental/referencemap.go:13 referenceMap
// PORT: Go stores `*collections.Set` pointers, and several files can share
// one set (`buildInfoToSnapshot` stores one set for each file id list). The
// port shares a set with `Arc`, so storing it does not copy its paths.

