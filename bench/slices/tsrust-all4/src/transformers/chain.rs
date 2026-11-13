//! Port of Go `transformers/chain.go`.

use crate::prelude::*;

use super::transformer::{};

// Go: transformers/chain.go:10 chainedTransformer



// Go: transformers/chain.go:38 Chain
// Chains transforms in left-to-right order, running them one at a time in order (as opposed to interleaved at each node)
// - the resulting combined transform only operates on SourceFile nodes
// PORT: Go returns a new factory. Rust closures cannot be named in a
// `const`, so this applies the chained factories to `opt` directly. A caller
// writes the Go factory as a named fn:
// `fn new_es2021_transformer(o: &TransformOptions) -> Option<TransformerBox> { chain(o, &[..]) }`.
