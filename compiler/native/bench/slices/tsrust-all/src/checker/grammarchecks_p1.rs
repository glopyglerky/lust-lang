//! Go `checker/grammarchecks.go` lines 1-951.

use crate::prelude::*;


// Go: checker/grammarchecks.go:58 getIdentifierFromEntityNameExpression


// PORT: Go `tspath.FileExtensionIsOneOf`. The `tspath` package has no shared
// Rust port, so this file keeps a private copy (like checker_p06/p17).
fn file_extension_is_one_of_gc1(path: &str, extensions: &[&str]) -> bool {
    for ext in extensions {
        // Go: tspath/path.go:1095 FileExtensionIs
        if path.len() > ext.len() && path.ends_with(ext) {
            return true;
        }
    }
    false
}

