//! Go: internal/execute/watchmanager/watchbackend.go (the fswatch backend
//! seam and the watch path filters).

use crate::execute::watchmanager::prelude::*;

use std::sync::Arc;

use crate::frontend::stringutil_ls;
use crate::frontend::tspath;
use crate::fswatch;

// Go: watchbackend.go:12 WatchBackend

// Go: watchbackend.go:17 WatchDirectoryRequest

// Go: watchbackend.go:26 CommandLineTestingWithWatchBackend

// Go: watchbackend.go:30 FSWatchBackend


// Go: watchbackend.go:72 ShouldIgnoreWatchPath
pub fn should_ignore_watch_path(path: &str) -> bool {
    let p = tspath::normalize_slashes(path);
    p.ends_with("/.git")
        || p.contains("/.git/")
        || p.contains("/node_modules/.")
        || p.contains("/.#")
}

// Go: watchbackend.go:80 CanWatchDirectory

// Go: watchbackend.go:90 PerceivedOsRootLengthForWatching
// PORT: Go `strings.EqualFold` is `stringutil_ls::equate_string_case_insensitive`
// (Go `stringutil.EquateStringCaseInsensitive` is `strings.EqualFold`).
