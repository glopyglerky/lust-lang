//! Go: internal/execute/watchmanager/watchbackend.go (the fswatch backend
//! seam and the watch path filters).

use crate::execute::watchmanager::prelude::*;

use std::sync::Arc;

use crate::frontend::stringutil_ls;
use crate::frontend::tspath;
use crate::fswatch;

// Go: watchbackend.go:12 WatchBackend
/// WatchBackend abstracts fswatch.Watcher for testing
///
/// PORT: Go returns an `io.Closer`; the port returns the fswatch
/// `Box<dyn fswatch::Watch>` (its `close` is `Close`). Go `ignore` is a
/// `func(string) bool` that can be nil; it runs on the fswatch debouncer
/// thread, so it is `Send + Sync`.
pub trait WatchBackend {
    fn watch_directory(
        &self,
        dir: &str,
        fn_: fswatch::WatchCallback,
        recursive: bool,
        ignore: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,
    ) -> Result<Box<dyn fswatch::Watch>, GoError>;
    /// PORT: Go takes a `[]WatchDirectoryRequest` slice; the port takes the
    /// requests by value, because a backend keeps their callbacks.
    fn watch_directories(
        &self,
        requests: Vec<WatchDirectoryRequest>,
    ) -> Result<Vec<Box<dyn fswatch::Watch>>, GoError>;
}

// Go: watchbackend.go:17 WatchDirectoryRequest
pub struct WatchDirectoryRequest {
    pub dir: String,
    pub callback: fswatch::WatchCallback,
    pub recursive: bool,
    pub ignore: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,
}

// Go: watchbackend.go:26 CommandLineTestingWithWatchBackend
/// CommandLineTestingWithWatchBackend is an optional extension of
/// [CommandLineTesting] that supplies a [WatchBackend] for test mode
pub trait CommandLineTestingWithWatchBackend {
    fn watch_backend(&self) -> Rc<dyn WatchBackend>;
}

// Go: watchbackend.go:30 FSWatchBackend
pub struct FsWatchBackend {
    pub inner: Arc<dyn fswatch::Watcher>,
}


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
