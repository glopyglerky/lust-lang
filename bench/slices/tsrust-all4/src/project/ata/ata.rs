//! Go `internal/project/ata/ata.go`.
//!
//! PORT: one thread (see `project/dirty/interfaces.rs`). `sync.Once` is an
//! `OnceState` cell, `atomic.Int32` a `Cell<i32>`, and
//! `collections.SyncMap` a `RefCell` map. `TypingsInstaller` methods take
//! `&self` like the Go pointer receiver. Go `logging.Logger` parameters are
//! `&dyn logging::Logger`, so a caller can pass `&Option<Rc<dyn Logger>>`
//! or `&Option<Rc<LogTree>>` (both implement the trait, as Go's nil-safe
//! loggers do). Go `vfs.FS` parameters are `&dyn vfs::Fs`.
//!
//! PORT: Go runs each ATA request on a goroutine that blocks while npm runs.
//! The port's request is a future (the Go functions that reach npm are
//! `async fn`, and each Go blocking point is an `.await`) that `run_task`
//! runs on the dispatch thread. A waiting request costs nothing until the
//! event it waits for wakes it, as a blocked goroutine does:
//! - npm: an executor that gives `npm_install_func` (the LSP server) runs
//!   npm on a helper thread, which posts the result to the dispatch thread
//!   once (`gostd::local::post_later`). Without it (the test mock) npm runs
//!   in the first poll and the whole request ends in that poll.
//! - `initOnce.Do`: the request that runs `init` wakes the others when it
//!   is done (`init_waiters`).
//! - the `concurrencySemaphore` send: a body that ends gives its slot to the
//!   oldest waiting body, as a Go channel receive does (`semaphore_waiters`).

use crate::project::ata::prelude::*;

use crate::frontend::core_ext::TypeAcquisition;
use crate::frontend::json::{self, JsonDecoder, JsonError, UnmarshalerFrom, };
use crate::frontend::json_ext::{};
use crate::frontend::{module, semver, tspath, vfs};
use crate::gostd::{self, Context, GoError};
use crate::project::logging::{self, Logger as _};
use std::cell::Cell;
use std::collections::VecDeque;
use std::future::{Future, poll_fn};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError};
use std::task::{Poll, Waker};

// Go: project/ata/ata.go:21 TypingsInfo
// PORT: the Go pointers are shared and read only, so they are `Rc`; nil is
// `None`. `UnresolvedImports *collections.Set[string]` is an
// `FxHashSet`.


// Go: project/ata/ata.go:33 CachedTyping
// PORT: Go `Version *semver.Version` is never nil here (every store takes
// the address of a parsed version), so the field is the value.
#[derive(Clone, Debug)]
pub struct CachedTyping {
    pub typings_location: String,
    pub version: semver::Version,
}

// Go: project/ata/ata.go:38 TypingsInstallerOptions
#[derive(Clone, Debug, Default)]
pub struct TypingsInstallerOptions {
    pub typings_location: String,
    pub throttle_limit: i32,
}

// Go: project/ata/ata.go:43 NpmExecutor
pub trait NpmExecutor {
    // PORT: Go returns `([]byte, error)` and reads the output when the error
    // is non-nil (installWorker logs it), so the result is a pair, not a
    // `Result`. `None` is Go's nil error.
    fn npm_install(&self, cwd: &str, args: &[String]) -> (Vec<u8>, Option<GoError>);

    /// PORT: no Go counterpart. A `Send` form of `npm_install` that ATA runs
    /// on a helper thread (`TypingsInstaller::npm_install`), or `None` to run
    /// `npm_install` on the calling thread.
    fn npm_install_func(&self) -> Option<NpmInstallFunc> {
        None
    }
}

/// PORT: the `Send` form of `NpmExecutor::npm_install`.
pub type NpmInstallFunc = Arc<dyn Fn(&str, &[String]) -> (Vec<u8>, Option<GoError>) + Send + Sync>;

/// PORT: Go `sync.Once` of `TypingsInstaller.initOnce`. `Running` while the
/// first `init` waits for npm; Go blocks the other callers of `Do` until the
/// body ends, and the port's callers wait for `Done`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnceState {
    NotStarted,
    Running,
    Done,
}

// Go: project/ata/ata.go:47 TypingsInstallerHost
// PORT: Go interfaces are structural, so every type that implements both
// traits is a host (blanket impl). `Rc<dyn TypingsInstallerHost>` upcasts to
// `Rc<dyn module::ResolutionHost>` for `module::new_resolver`.
pub trait TypingsInstallerHost: NpmExecutor + module::ResolutionHost {}

impl<T: NpmExecutor + module::ResolutionHost> TypingsInstallerHost for T {}

// Go: project/ata/ata.go:52 TypingsInstaller
// PORT: `packageNameToTypingLocation` is ranged in DiscoverTypings, so it is
// an `IndexMap` (insertion order; Go map order is random). A
// `typesRegistry` entry is `Option`: Go keeps a JSON `null` entry as a nil
// map, and DiscoverTypings tests `registryEntry != nil`. The Go
// `concurrencySemaphore chan struct{}` is the `sync_channel` pair
// (PORTING "Go runtime"), and `semaphore_waiters` holds the bodies that
// block in a send on it.

// Go: project/ata/ata.go:67 NewTypingsInstaller

// Go: project/ata/ata.go:75 ProjectID (ts#64319)
// PORT: Go `interface { fmt.Stringer }`.
pub trait ProjectID {
    fn string(&self) -> String;
}


// Go: project/ata/ata.go:92 tsVersionToUse
// !!! sheetal currently we use latest instead of core.VersionMajorMinor()
pub const TS_VERSION_TO_USE: &str = "latest";

// Go: project/ata/ata.go:94 TypingsInstallRequest
// PORT: Go `TypingsInfo *TypingsInfo` is never nil (the session passes the
// address of a local), so it is an `Rc`. `GetScriptKind` is a stored func.

// Go: project/ata/ata.go:106 TypingsInstallResult
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypingsInstallResult {
    pub typings_files: Vec<String>,
    pub files_to_watch: Vec<String>,
}


/// PORT: the result of `NpmExecutor::npm_install`.
type NpmResult = (Vec<u8>, Option<GoError>);

// Go: project/ata/ata.go:284 installNpmPackages
// PORT: the Go test entry; it runs `install_npm_packages_async` to its end
// with `install_packages` as each body.

// Go: project/ata/ata.go:284 installNpmPackages
// PORT: each `tg.Go` body is a future. The bodies run at the same time, each
// while it holds a semaphore slot, and the result is the first error, as
// `tg.Wait()` gives.





/// PORT: the waker of an ATA request. A wake queues the next poll of the
/// request on the dispatch thread (`gostd::local::go`), once until that poll
/// runs. Every wake comes on the dispatch thread: an npm result arrives
/// through `gostd::local::post_later`, and `init` and the semaphore wake
/// from other requests.
struct TaskWaker {
    id: u64,
    thread: std::thread::ThreadId,
    queued: AtomicBool,
}






// Go: project/ata/ata.go:395 npmConfig
// PORT: Go `map[string]any` is `IndexMap<String, LspAny>` (PORTING "JSON");
// nil is `None`. processCacheLocation ranges over it: insertion order, where
// Go map order is random.

// Go: project/ata/ata.go:399 npmDependecyEntry
#[derive(Clone, Debug, Default)]
pub struct NpmDependecyEntry {
    pub version: String,
}

// Go: project/ata/ata.go:402 npmLock
// PORT: nil maps are `None` (processCacheLocation tests them for nil).
#[derive(Clone, Debug, Default)]
pub struct NpmLock {
    pub dependencies: Option<FxHashMap<String, NpmDependecyEntry>>,
    pub packages: Option<FxHashMap<String, NpmDependecyEntry>>,
}

// PORT: Go decodes npmConfig, npmDependecyEntry and npmLock by reflection
// with the JSON v2 default struct rules (PORTING "JSON"): input names match
// exactly, unknown names are skipped, `null` sets the zero value. The Go
// `map[string]map[string]string` (types registry) uses the generic
// `FxHashMap<String, V>` and `Option<T>` impls in `frontend/json.rs` and
// `frontend/json_ext.rs`, so it has no impl here.




// Go: project/ata/ata.go:466 parseNpmConfigOrLock
// PORT: Go `[T npmConfig | npmLock]` is any `UnmarshalerFrom`. Like Go, a
// read or decode error is ignored and a partly decoded `config` is kept.


#[cfg(test)]
mod npm_thread_tests {
    use super::*;
    use std::sync::{Condvar, Mutex};

    /// An ATA host whose npm runs on a helper thread and waits for `open`.
    struct GatedNpm {
        fs: Rc<dyn vfs::Fs>,
        cwd: String,
        calls: Arc<Mutex<Vec<String>>>,
        gate: Arc<(Mutex<bool>, Condvar)>,
    }

    impl NpmExecutor for GatedNpm {
        fn npm_install(&self, _cwd: &str, _args: &[String]) -> (Vec<u8>, Option<GoError>) {
            panic!("npm ran on the dispatch thread");
        }

        fn npm_install_func(&self) -> Option<NpmInstallFunc> {
            let (calls, gate) = (self.calls.clone(), self.gate.clone());
            Some(Arc::new(move |cwd: &str, args: &[String]| {
                calls.lock().unwrap().push(args.join(" "));
                let (open, cond) = &*gate;
                drop(
                    cond.wait_while(open.lock().unwrap(), |open| !*open)
                        .unwrap(),
                );
                let dir = std::path::Path::new(cwd).join("node_modules/types-registry");
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(dir.join("index.json"), r#"{"entries":{"left-pad":{}}}"#).unwrap();
                (Vec::new(), None)
            }))
        }
    }

    impl module::ResolutionHost for GatedNpm {
        fn fs(&self) -> &dyn vfs::Fs {
            &*self.fs
        }
        fn get_current_directory(&self) -> &str {
            &self.cwd
        }
    }

    fn gated_installer(throttle_limit: i32) -> (Rc<GatedNpm>, Rc<TypingsInstaller>, String) {
        let dir = std::env::temp_dir().join(format!(
            "goport-ata-{}-{throttle_limit}",
            std::process::id()
        ));
        let cwd = dir.to_string_lossy().replace('\\', "/");
        let host = Rc::new(GatedNpm {
            fs: vfs::osvfs::osvfs_fs(),
            cwd: cwd.clone(),
            calls: Arc::default(),
            gate: Arc::default(),
        });
        let ti = new_typings_installer(
            &TypingsInstallerOptions {
                typings_location: format!("{cwd}/cache"),
                throttle_limit,
            },
            host.clone(),
        );
        (host, ti, cwd)
    }

    /// A task that counts its polls in `polls`.
    fn counted(
        polls: Rc<Cell<u32>>,
        fut: impl Future<Output = ()> + 'static,
    ) -> Pin<Box<dyn Future<Output = ()>>> {
        let mut fut = Box::pin(fut);
        Box::pin(poll_fn(move |cx| {
            polls.set(polls.get() + 1);
            fut.as_mut().poll(cx)
        }))
    }

    /// Opens the gate of `host` and runs this thread's work until `done`
    /// is `n`.
    fn open_and_run(host: &GatedNpm, done: &Cell<usize>, n: usize) {
        let (open, cond) = &*host.gate;
        *open.lock().unwrap() = true;
        cond.notify_all();
        while done.get() < n && gostd::local::wait_pending() {
            gostd::local::run_pending();
        }
        assert_eq!(done.get(), n);
    }

    /// Go runs ATA requests on goroutines, and `initOnce.Do` blocks a second
    /// request until the first one's npm call ends. The port's requests wait
    /// for npm without blocking the thread, and npm runs once. Neither
    /// request is polled again until npm ends: the first is woken by the npm
    /// post, the second by the end of `init`.
    #[test]
    fn second_request_waits_for_init_off_thread() {
        let (host, ti, cwd) = gated_installer(5);
        let done = Rc::new(Cell::new(0));
        let polls: Vec<Rc<Cell<u32>>> = (0..2).map(|_| Rc::default()).collect();
        for polls in &polls {
            let (ti, fs, done) = (ti.clone(), host.fs.clone(), done.clone());
            run_task(counted(polls.clone(), async move {
                ti.init("p", &*fs, &None::<Rc<dyn logging::Logger>>).await;
                assert_eq!(ti.init_once.get(), OnceState::Done);
                done.set(done.get() + 1);
            }));
        }
        assert_eq!((done.get(), ti.init_once.get()), (0, OnceState::Running));
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert!(
            !gostd::local::has_pending(),
            "a request was woken before npm ended"
        );

        open_and_run(&host, &done, 2);
        assert_eq!(polls.iter().map(|p| p.get()).collect::<Vec<_>>(), [2, 2]);
        assert_eq!(
            *host.calls.lock().unwrap(),
            ["install --ignore-scripts types-registry@latest"]
        );
        assert!(ti.types_registry.borrow().contains_key("left-pad"));
        let _ = std::fs::remove_dir_all(&cwd);
    }

    /// Go `ThrottleGroup.Go` blocks in a send on the full semaphore channel,
    /// and a body that ends gives its slot to the oldest blocked body. With
    /// one slot, three npm calls run one at a time in request order, and a
    /// waiting body is polled only when it gets the slot and when its npm
    /// call ends.
    #[test]
    fn semaphore_gives_freed_slots_in_order() {
        let (host, ti, cwd) = gated_installer(1);
        let done = Rc::new(Cell::new(0));
        let polls: Vec<Rc<Cell<u32>>> = (0..3).map(|_| Rc::default()).collect();
        for (i, polls) in polls.iter().enumerate() {
            let (ti, cwd, done) = (ti.clone(), cwd.clone(), done.clone());
            run_task(counted(polls.clone(), async move {
                let args = [format!("t{i}")];
                let npm = ti.npm_install(&cwd, &args);
                let (_, err) =
                    throttled(&ti.concurrency_semaphore, &ti.semaphore_waiters, npm).await;
                assert!(err.is_none());
                done.set(done.get() + 1);
            }));
        }
        let started = std::time::Instant::now();
        while host.calls.lock().unwrap().is_empty() && started.elapsed().as_secs() < 10 {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert_eq!(*host.calls.lock().unwrap(), ["t0"]);
        assert_eq!(ti.semaphore_waiters.borrow().len(), 2);
        assert!(
            !gostd::local::has_pending(),
            "a request was woken before npm ended"
        );

        open_and_run(&host, &done, 3);
        assert_eq!(*host.calls.lock().unwrap(), ["t0", "t1", "t2"]);
        assert_eq!(polls.iter().map(|p| p.get()).collect::<Vec<_>>(), [2, 3, 3]);
        assert!(ti.semaphore_waiters.borrow().is_empty());
        assert!(
            ti.concurrency_semaphore.1.try_recv().is_err(),
            "a slot stayed taken"
        );
        let _ = std::fs::remove_dir_all(&cwd);
    }
}
