//! Go `internal/project/client.go`.
//!
//! PORT: Go `Client` is an interface; here it is a trait used as
//! `Rc<dyn Client>` (a Go nil client is `None` where Go checks for nil).
//! The LSP server (w6) implements it on the dispatch thread.

use crate::project::prelude::*;

// Go: project/client.go:11 Client
// PORT: `watchers []*lsproto.FileSystemWatcher` is `&[..]` (PORTING
// "Types"). `params *lsproto.PublishDiagnosticsParams` is passed by value,
// because the server moves it into the outgoing notification. `args ...any`
// is `Vec<String>` (built with `args![..]`).
