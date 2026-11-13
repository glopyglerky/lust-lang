//! Port of Go `checker/nodebuilderscopes.go`: signature scopes for the node
//! builder.

use crate::prelude::*;

// Go: checker/nodebuilderscopes.go:10 cloneNodeBuilderContext
// PORT: the restore closure keeps its own handle to the context and puts the
// saved copy-on-write state back.

// Go: checker/nodebuilderscopes.go:35 localsRecord

// PORT: private accessor, see `p1_ctx` in nodebuilder_impl_p1.rs.


