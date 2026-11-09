//! Go: execute/build/orchestrator.go:264-658, the watch part of the build
//! orchestrator (`Watch`, `updateWatch`, `resetCaches`,
//! `checkTasksForEventChanges`, `computeDesiredWatches`, `DoCycle`), with
//! `rangeTask` (orchestrator.go:688) for these callers, and
//! execute/build/buildtask.go:825-844 (`updateWatch`, `resetConfig`).
//!
//! PORT: Go runs `DoCycle` from `WatchManager.RunLoop` on the goroutine
//! that called `Watch`. The port does the same on the orchestrator thread.
//! The watch manager is `Rc<RefCell<WatchManager>>` (see orchestrator.rs):
//! the loop keeps a shared borrow while `do_cycle` borrows the orchestrator
//! mutably, and `do_cycle` only takes shared borrows of the manager.
//!
//! PORT: Go `task.built` and `task.done` channels are dropped (see
//! build_task.rs), so the lines that make new ones are left out. Go
//! `atomic.Bool` flags on one goroutine are plain `bool`s.
//!
//! PORT: each project compiles in this process (build_task.rs). A watch
//! cycle makes new program versions, and each is released when its task
//! is built (orchestrator.rs). The old program is read from build info, as Go does in build
//! mode (buildtask.go:251).

use crate::execute::build::host::ModuleIndicatorInputs;
use crate::execute::tsc::compile::{Watcher, write_str};
use crate::execute::watchmanager::{DirWatchSet, new_dir_watch_set};
use crate::frontend::prelude::*;
use crate::fswatch;
use crate::gostd::Context;
use std::sync::PoisonError;
use std::time::SystemTime;


// Go: build/orchestrator.go:85 `var _ tsc.Watcher = (*Orchestrator)(nil)`

