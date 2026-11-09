//! Port of Go `sourcemap/util.go`.

use crate::prelude::*;

use super::lineinfo::ECMALineInfo;

// Go: sourcemap/util.go:11 TryGetSourceMappingURL
// Tries to find the sourceMappingURL comment at the end of a file.
