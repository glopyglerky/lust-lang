//! Go `internal/core/context.go`: request ids and checker lifetimes carried
//! in a `Context`.

use crate::frontend::prelude::*;

use crate::gostd::context::{self, ContextKey};

// Go: core/context.go:7 key
// Go: core/context.go:10 requestIDKey

// Go: core/context.go:11 checkerLifetimeKey

// Go: core/context.go:14 WithRequestID

// Go: core/context.go:18 GetRequestID

// Go: core/context.go:25 CheckerLifetime
/// Go `type CheckerLifetime int`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CheckerLifetime(pub i32);

impl CheckerLifetime {
    // Go: core/context.go:28 CheckerLifetimeTemporary
    pub const TEMPORARY: CheckerLifetime = CheckerLifetime(0);
    // Go: core/context.go:29 CheckerLifetimeDiagnostics
    pub const DIAGNOSTICS: CheckerLifetime = CheckerLifetime(1);
    // Go: core/context.go:30 CheckerLifetimeAPI
    pub const API: CheckerLifetime = CheckerLifetime(2);
}

// Go: core/context.go:33 WithCheckerLifetime

// Go: core/context.go:37 GetCheckerLifetime
