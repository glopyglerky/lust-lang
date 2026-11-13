//! Port of Go `checker/relater.go` lines 916-1826: best-match helpers for
//! error elaboration, discriminants, variances and signature relations.

use crate::prelude::*;


// PORT: Go interface `Discriminator`. The Go implementations hold a
// `*Checker`; here the checker is passed to `matches` instead. Go `int`
// indexes are `i32`.
// Go: checker/relater.go:1199 Discriminator

// PORT: Go `TypeDiscriminator{c, props, isRelatedTo}`. The `c` field is
// dropped (the checker is passed to each method). `isRelatedTo` is the
// borrowed Go func param.
// Go: checker/relater.go:1030 TypeDiscriminator


