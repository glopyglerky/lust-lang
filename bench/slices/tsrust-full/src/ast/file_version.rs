//! Rust-only: the owner and liveness of freeable file versions (lsshells
//! M3a, M3b, M3c).
//!
//! Go frees an `*ast.SourceFile` when no program and no parse cache entry
//! holds it (project/snapshot.go:537 `dispose`: `parseCache.Deref` for each
//! file of a released program). Here a static published file store is
//! leaked (`ast/store.rs`), so its id stays readable. A `FileVersion` gives
//! a file version the Go lifetime:
//! - its parse holds it (`ParsedSourceFile::version`), so each frontend
//!   program (`NewProgram`) and each parse cache entry that has the parse
//!   holds it;
//! - the tables of each program version that has the file hold it
//!   (`program::VersionTables`), so a checker, bind, emit or search thread
//!   that was seeded from that version holds it too (`WorkerSeed`);
//! - a thread that reads it pins it (`with_file_version`, `PINS`) until the
//!   next program release, or the next release of a source file lease that
//!   held the last parse holder (`release_file_version_pins`), or its end;
//! - a `FileRef` guard holds it while the guard lives;
//! - the registry here keeps only a `Weak`.
//!
//! M3b: at publish the version takes its `FileStore` and its `GoFile`
//! (info, `node_bind`, `file_bind`, `flow_nodes`)
//! (`ast::store::VersionStore`). Its node records and kids are in its node
//! shell, the registry block of its id, so the header and child reads stay
//! inline (`ast::store::node_shell`); its foreign parents are leaked there,
//! and the child link column is dropped. AST node records, step 4: the
//! records and kids are in a pooled block that the version gives back when
//! it dies; after two more pin releases (`pin_epoch`: program releases, and
//! lease releases that free a parse) a later node shell can take it
//! (`ast::store::BlockPool`).
//! M3c (owned nodes; on by default in a language server or API process,
//! see `owned_nodes_enabled`): its parse was a freeable parse
//! (`ast::enter_freeable_parse`), so its store owns its astdata nodes (node
//! structs and data boxes), its
//! pending lists and its parse lists (`ast::store::OwnedAst`), and they go
//! with the version. The node shell has no node column: a node data read
//! of the file is a scoped read of the pinned version
//! (`ast::with_scoped_store_node`), and a list of its node data is a handle
//! (`ast::StoreList`) that is read at each use.
//! M3d: the binder lineage binds the version into symbol and table chunks
//! of its own. After the version dies, the next bind frees them
//! (`program::Lineage`), and each program copy of the lineage lets go of
//! them when its release frees its tables (M2c, `program::bound_symbols`).
//! When the last holder lets go, the version is dead: its id goes to
//! `DEAD_FILES` and leaves the registry, its store and `GoFile` are freed
//! (after it, on a free thread, when a pin release of the stdio API or
//! `tsc -b -w` takes its data: `take_data`), and each per-file thread-local
//! map (`PerFileMap`) forgets the entries of that id when it is next
//! written. A later read of its store, `GoFile` or node data panics
//! with "file version N is released": ids are never reused, so a missed
//! holder panics there and never reads another file. A header or child
//! read of its node shell reads its pooled block: the data of that node
//! until another version takes the block, then that version's data. With
//! debug assertions that read panics with the same message (the owner
//! check, `ast::store::file_block`). A release build checks the owner only
//! in the binder field reads (`check_block_owner`), so there a stale
//! header or child read gives wrong data and no panic. The check in every
//! `file_block` read (one load, compare and branch) costs too much for a
//! release build (watchfix1 item 5, instructions:u of stable builds):
//! `goport -p --singleThreaded` query +7.9%, hono +7.9%, zod +7.1%, effect
//! +10.2% (+6.7% to +9.7% on the default threads), and the editor long
//! sessions query-core +9.6%, hono +4.5%, effect +9.3%. A correct reader
//! never sees a reuse: every holder of a node holds its version, and a
//! given-back block waits two pin releases (`BlockPool`). No standing run
//! has debug assertions: the protected tests run `--release`
//! (`build-goport-tests.sh`), and the corpus, editor and oracle runs use
//! release bins. A debug test build checks each `file_block` read.
//!
//! Freeable rule (`free_file_versions`, `freeable_path`): only a parse of
//! a path that a publish on this thread published before, in a language
//! server or API process (the parse cache, project/parsecache.rs) or a
//! `tsc --watch` or `tsc -b --watch` process (each build,
//! `program::mark_freeable_parses`; watchfree1). An API source file lease
//! notes its path before its parse (apimem1 fix C,
//! `project::SnapshotHost::acquire_source_file`), so the first version of a
//! leased path gets one. So the first publish, the first version of each
//! file (but a leased one) and every other CLI publish never get a
//! `FileVersion`.
//! `GOPORT_FREE_FILE_VERSIONS=0` turns it off (the behavior before M3a);
//! `=1` turns it on in any process, and then
//! `program::update_program_version` (`goport_multiprog`) applies the same
//! rule to its new parses (the compiler host opens the freeable parse scope
//! for them, M3c). A parse that a parse worker made (prefetch) keeps its
//! nodes in the leaked AST arena; its version still frees its store and
//! `GoFile`. Watch mode parses ahead in its first build and after a config
//! change or an overflow; there a worker parse of a published path owns
//! its nodes (`CompilerHost::freeable_worker_parses`, watchcfg1).

use crate::prelude::*;
use std::cell::Cell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError, Weak};






/// The ids of the dead file versions, in the order they died. `PerFileMap`
/// reads the ids after the length it saw last.
// PERF: one id per edit, so it grows by 8 bytes per edit.
static DEAD_FILES: Mutex<Vec<usize>> = Mutex::new(Vec::new());

/// The length of `DEAD_FILES` (the dead-file epoch). A map that saw this
/// length has nothing to forget.
static DEAD_COUNT: AtomicUsize = AtomicUsize::new(0);

/// The number of file versions made in this process.
static MADE: AtomicUsize = AtomicUsize::new(0);

/// Set by `project::new_session`: this process runs the language server or
/// the API.
static EDITOR_PROCESS: AtomicBool = AtomicBool::new(false);

/// Set by `Watcher::start` and `Orchestrator::start`: this process runs
/// `tsc --watch`, with or without `--build`.
static WATCH_PROCESS: AtomicBool = AtomicBool::new(false);

/// Set when the first freeable version is published. Until then a registry
/// read (`with_version_store` in `ast/store.rs`) never looks for a version.
static FREEABLE_PUBLISHED: AtomicBool = AtomicBool::new(false);

/// Raised by each pin release (`release_file_version_pins`). A thread
/// whose pins are from an older epoch drops them at its next pinned read.
static PIN_EPOCH: AtomicUsize = AtomicUsize::new(0);


/// Marks this process as a language server or API process, where
/// `free_file_versions` and owned nodes are on by default.
/// `project::new_session` and `api::new_standalone_session` call it.
pub fn set_editor_process() {
    EDITOR_PROCESS.store(true, Ordering::Relaxed);
}

/// Marks this process as a `tsc --watch` process (with or without
/// `--build`), where `free_file_versions` and owned nodes are on by
/// default, as in a language server. `Watcher::start` and
/// `Orchestrator::start` call it before the first build.
// PORT: not in Go. Go frees an old `*ast.SourceFile` of a watch rebuild
// with its GC (execute/watcher.go:482, :567). Every tsc command of the
// go_baselines tests runs in a child process of its own, so the flag never
// reaches another test.
pub fn set_watch_process() {
    WATCH_PROCESS.store(true, Ordering::Relaxed);
}

/// True in a process where `free_file_versions` and owned nodes are on by
/// default: a language server or API process (`set_editor_process`) or a
/// `tsc --watch` process (`set_watch_process`).
pub(crate) fn frees_file_versions_by_default() -> bool {
    EDITOR_PROCESS.load(Ordering::Relaxed) || WATCH_PROCESS.load(Ordering::Relaxed)
}

/// True when this process frees the file versions that it publishes again.
/// `GOPORT_FREE_FILE_VERSIONS` is read once: `0` is off, `1` is on; else it
/// is on in a language server, API or `tsc --watch` process only
/// (`frees_file_versions_by_default`).
pub fn free_file_versions() -> bool {
    static FLAG: OnceLock<Option<bool>> = OnceLock::new();
    let flag = *FLAG.get_or_init(
        || match std::env::var("GOPORT_FREE_FILE_VERSIONS").as_deref() {
            Ok("0") => Some(false),
            Ok("1") => Some(true),
            _ => None,
        },
    );
    flag.unwrap_or_else(frees_file_versions_by_default)
}

thread_local! {
    /// The paths that a publish on this thread published, while
    /// `free_file_versions` is on (`note_published_path`). Shared with the
    /// parse workers of a load (`published_paths`); a new path copies the
    /// set while a load holds it.
    static PUBLISHED_PATHS: RefCell<Arc<FxHashSet<String>>> = RefCell::default();
}



/// The freeable rule: true when a new parse of `path` gets a
/// `FileVersion`, because `free_file_versions` is on and a publish on this
/// thread published `path` before.
pub(crate) fn freeable_path(path: &str) -> bool {
    free_file_versions() && PUBLISHED_PATHS.with(|paths| paths.borrow().contains(path))
}



/// True once a freeable version is published in this process. A CLI
/// process never sets it, so its registry reads never look for a version.
#[inline]
pub(crate) fn any_freeable_published() -> bool {
    FREEABLE_PUBLISHED.load(Ordering::Acquire)
}


/// The pin epoch: the number of pin releases so far
/// (`release_file_version_pins`). A pooled node block given back at epoch
/// `e` is free from epoch `e + 2` (`ast::store::BlockPool`).
#[inline]
pub(crate) fn pin_epoch() -> usize {
    PIN_EPOCH.load(Ordering::Acquire)
}

/// Panics for a read of dead file version `file`.
#[cold]
#[inline(never)]
pub(crate) fn released(file: usize) -> ! {
    panic!("file version {file} is released")
}














#[cold]
#[inline(never)]
fn no_hot_version() -> ! {
    panic!("no hot file version on this thread")
}






thread_local! {
    /// True while a `PinRelease` of this thread waits for its drop
    /// (`release_file_version_pins_later`).
    static PIN_RELEASE_QUEUED: Cell<bool> = const { Cell::new(false) };
    /// `Some(min_nodes)` on a thread whose `PinRelease` frees the data of a
    /// dying version of `min_nodes` or more nodes on the free thread
    /// (`free_released_versions_in_background`).
    static FREE_IN_BACKGROUND: Cell<Option<usize>> = const { Cell::new(None) };
}

/// The node count from which the free of a dying version's data goes to the
/// free thread (`free_released_versions_in_background`): about a 60 KB file.
// PERF: (apiperf1) mini-743d. The free of a version of a 260 KB file (about
// 70,000 nodes) took about 1 ms of each releaseSourceFile. For small texts
// (about 150 nodes) the free thread made a loop of 3,000 leases 3 to 6%
// slower: the next parse reused memory that the other thread freed.
const BACKGROUND_FREE_MIN_NODES: usize = 10_000;

/// `release_file_version_pins` when the value drops.
struct PinRelease;




/// Not in Go (perf, apiperf1): on this thread, the pin release of a source
/// file lease release (`release_file_version_pins_later`) gives the data of
/// each version that dies in it with `BACKGROUND_FREE_MIN_NODES` or more
/// nodes to the free thread (`free_in_background`), so it is freed there,
/// beside the next requests, as Go's GC frees the leased
/// `*ast.SourceFile`. The version itself dies in the release: its id is
/// dead before the send, as with a free in the release. The stdio API
/// server calls it with `gostd::local::keep_garbage`. Elsewhere the free
/// runs in the release, so a test sees the data go at `drop_garbage`.
pub fn free_released_versions_in_background() {
    FREE_IN_BACKGROUND.with(|min_nodes| min_nodes.set(Some(BACKGROUND_FREE_MIN_NODES)));
}









/// `FileRef` to `$part` of the `GoFile` of published file `$file` (a file
/// id). `$part` is a place expression of `$g` (the `GoFile`) and `$k` (the
/// value of `$key`; name it `_key` when `$part` does not use it). It is
/// written once and used for both variants, so it must not capture anything
/// else. Panics when `$file` is not published.
// PORT: a `FileRef::Pinned` getter must be a plain function, so the part is
// repeated in a non-capturing closure.
macro_rules! go_file_ref {
    ($file:expr, $key:expr, |$g:ident, $k:ident| $part:expr) => {{
        let key: usize = $key;
        match $crate::ast::go_file($file) {
            $crate::ast::FileRef::Static($g) => {
                let $k = key;
                $crate::ast::FileRef::Static(&$part)
            }
            $crate::ast::FileRef::Pinned { version, .. } => $crate::ast::FileRef::Pinned {
                version,
                key,
                get: |version, $k| {
                    let $g = version.go_file();
                    &$part
                },
            },
        }
    }};
}















/// The number of file versions made in this process.
#[must_use]
pub fn file_versions_made() -> usize {
    MADE.load(Ordering::Relaxed)
}

/// The number of dead file versions in this process.
#[must_use]
pub fn dead_file_versions() -> usize {
    DEAD_COUNT.load(Ordering::Acquire)
}


/// A thread-local map with per-file entries: each key is a node, and the
/// entry belongs to the file of that node. It forgets the entries of dead
/// file versions when it is next written (`write`). A read (`Deref`) can
/// still find such an entry before that; its value is owned by the map, so
/// it is still valid, but a key of a dead version is never read again.
#[derive(Clone, Debug)]
pub(crate) struct PerFileMap<V> {
    /// `DEAD_COUNT` when the map last forgot dead entries.
    seen: usize,
    map: FxHashMap<Node, V>,
    /// The ids of the dead file versions that the map knows of (the first
    /// `seen` of `DEAD_FILES`), so a miss can tell a node of a dead version
    /// (`is_dead`).
    // PERF: one id per edit.
    dead: FxHashSet<usize>,
}

impl<V> PerFileMap<V> {

    /// True when `node` is a node of a dead file version. Call it after
    /// `write`, which brings the dead ids up to date.
    // PERF: no hash in a process that frees no file version.
    #[inline]
    pub(crate) fn is_dead(&self, node: Node) -> bool {
        !self.dead.is_empty() && self.dead.contains(&node.file_index())
    }


}


impl<V> Deref for PerFileMap<V> {
    type Target = FxHashMap<Node, V>;

    fn deref(&self) -> &FxHashMap<Node, V> {
        &self.map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The first node of file `file`. The tests with a fixed id (`DYING`,
    /// `OTHER`) do not publish it; the others publish the file first
    /// (`publish_pinned`).
    fn node(file: usize) -> Node {
        Node(((file as u64) << 32) | 1)
    }

    // A map keeps the entries of a dead version until its next write, then
    // forgets them and keeps the entries of other files.
    #[test]
    fn per_file_map_forgets_dead_file_versions() {
        const DYING: usize = (1 << 22) - 1;
        const OTHER: usize = DYING - 1;
        let mut map = PerFileMap::new();
        map.write().insert(node(DYING), 1);
        map.write().insert(node(OTHER), 2);
        let version = FileVersion::new(DYING);
        let probe = file_version_probe(node(DYING)).expect("the registry has the version");

        drop(version);
        assert!(probe.is_freed());
        assert!(file_version_probe(node(DYING)).is_none());
        assert_eq!(map.get(&node(DYING)), Some(&1));
        map.write();
        assert_eq!(map.get(&node(DYING)), None);
        assert_eq!(map.get(&node(OTHER)), Some(&2));
    }

    /// Holds the free thread until its sender sends or drops.
    struct Blocker(std::sync::mpsc::Receiver<()>);

    impl Drop for Blocker {
        fn drop(&mut self) {
            let _ = self.0.recv();
        }
    }

    /// Starts the free thread when it has not started and blocks it: the
    /// values sent to it after this wait until the returned sender sends or
    /// drops.
    fn block_free_thread() -> std::sync::mpsc::Sender<()> {
        let (go, wait) = std::sync::mpsc::channel();
        free_in_background(Box::new(Blocker(wait)));
        go
    }

    /// Publishes on this thread a freeable version of a file of one
    /// identifier per name and pins it. Gives the version (its other
    /// holder) and a weak handle to its text, which goes with its data (its
    /// store).
    fn publish_pinned(name: &'static str, names: &[&str]) -> (Arc<FileVersion>, Weak<str>) {
        let text = FileText::Shared(Arc::from(names.join(";")));
        let weak = text.weak().expect("a shared text");
        let file = super::super::store::new_file_store(name, text);
        let factory = crate::ast::NodeFactory::for_file(file);
        for identifier in names {
            factory.new_identifier(*identifier);
        }
        super::super::store::freeze_file_store(file);
        let version = FileVersion::new(file);
        crate::program::publish_parsed_files("/");
        assert!(version.node_count() > 0, "the version is published");
        drop(pinned_file_version(file).expect("the version is live"));
        (version, weak)
    }

    // apiperf1, followups30 item 3 (a): on a thread that frees in the
    // background (the stdio API server), a version dies in the pin release
    // of a lease release, also when the free thread is busy: its id is dead
    // and a read of it panics. The data of a version of `min_nodes` or more
    // nodes goes to the free thread, which frees it after the values queued
    // before it. The data of a smaller version is freed in the release. A
    // version with another holder lives until that holder lets go. It
    // publishes, so no other test may build or publish stores while it runs
    // (the runner uses one thread).
    #[test]
    fn background_pin_release_frees_on_the_free_thread() {
        // Its own thread, so the background mode stays there.
        std::thread::spawn(|| {
            let (small_version, small_text) = publish_pinned("/fv/small.ts", &["a"]);
            let (big_version, big_text) = publish_pinned("/fv/big.ts", &["a", "b", "c", "d"]);
            let (kept, kept_text) = publish_pinned("/fv/kept.ts", &["a", "b", "c", "d"]);
            let big_nodes = big_version.node_count();
            assert!(small_version.node_count() < big_nodes);
            let [small, big] = [&small_version, &big_version].map(|version| version.file());
            let probes = [small, big, kept.file()].map(|file| {
                file_version_probe(node(file)).expect("the pin of this thread holds the version")
            });
            // Only the pins of this thread hold small and big.
            drop((small_version, big_version));
            let go = block_free_thread();
            FREE_IN_BACKGROUND.with(|min_nodes| min_nodes.set(Some(big_nodes)));
            let dead = dead_file_versions();
            release_file_version_pins_later();
            assert!(
                probes[..2].iter().all(FileVersionProbe::is_freed),
                "the versions die in the release"
            );
            let (died, _) = dead_files_since(dead);
            assert!(died.contains(&small) && died.contains(&big), "{died:?}");
            assert!(!died.contains(&kept.file()), "{died:?}");
            assert!(!probes[2].is_freed(), "its other holder keeps a version");
            drop(kept);
            assert!(probes[2].is_freed(), "the last holder frees the version");
            assert!(
                kept_text.upgrade().is_none(),
                "the last holder frees the data"
            );
            let read = std::panic::catch_unwind(|| pinned_file_version(big).is_some());
            let message = read.expect_err("a read of a dead version panics");
            assert_eq!(
                message.downcast_ref::<String>(),
                Some(&format!("file version {big} is released"))
            );
            assert!(
                small_text.upgrade().is_none(),
                "the release frees the data of a small version"
            );
            assert!(
                big_text.upgrade().is_some(),
                "the data of a big version waits for the free thread"
            );
            go.send(()).expect("the free thread waits");
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
            while big_text.upgrade().is_some() {
                assert!(
                    std::time::Instant::now() < deadline,
                    "the free thread frees the data"
                );
                std::thread::yield_now();
            }
        })
        .join()
        .expect("the test thread ends");
    }

    // followups30 item 3 (b): a `PinRelease` that waits in the garbage of
    // `gostd::local` when its thread ends drops its pins there, so the
    // thread's end frees the data and does not send it to the free thread
    // (a thread-local destructor must not start that thread). It publishes,
    // so no other test may build or publish stores while it runs (the
    // runner uses one thread).
    #[test]
    fn a_pin_release_at_the_thread_end_frees_in_place() {
        // First, so the free thread exists and the end does not start it.
        let go = block_free_thread();
        let (probe, text) = std::thread::spawn(|| {
            // `PINS` before `LOCAL` (`keep_garbage`): the thread-local
            // destructors run in the reverse order of the first uses, so
            // `LOCAL` drops the release while the pins are still there.
            PINS.with(|_| ());
            let (version, text) = publish_pinned("/fv/end.ts", &["a"]);
            let file = version.file();
            drop(version);
            crate::gostd::local::keep_garbage();
            FREE_IN_BACKGROUND.with(|min_nodes| min_nodes.set(Some(0)));
            release_file_version_pins_later();
            assert_eq!(
                crate::gostd::local::garbage_len(),
                1,
                "the release waits for `drop_garbage`"
            );
            let probe = file_version_probe(node(file)).expect("the pin holds the version");
            // The thread ends with no `drop_garbage`.
            (probe, text)
        })
        .join()
        .expect("the test thread ends");
        assert!(probe.is_freed(), "the end of the thread frees the version");
        assert!(
            text.upgrade().is_none(),
            "the end of the thread frees the data, not the free thread"
        );
        go.send(()).expect("the free thread waits");
    }

    // followups32 (followups30 item 3 (a)): a version that dies in a pin
    // release on a thread that frees in the background leaves the registry
    // in the release (`FileVersion::drop`), before its data goes to the free
    // thread, so a busy free thread leaves no entry of a dead version. It
    // publishes, so no other test may build or publish stores while it runs
    // (the runner uses one thread).
    #[test]
    fn a_pin_release_takes_a_dying_version_out_of_the_registry() {
        std::thread::spawn(|| {
            let (version, text) = publish_pinned("/fv/registry.ts", &["a"]);
            let file = version.file();
            drop(version);
            assert!(lock(&VERSIONS).contains_key(&file), "a live version");
            let go = block_free_thread();
            FREE_IN_BACKGROUND.with(|min_nodes| min_nodes.set(Some(0)));
            release_file_version_pins_later();
            assert!(
                !lock(&VERSIONS).contains_key(&file),
                "the registry has no entry of the dead version"
            );
            assert!(
                text.upgrade().is_some(),
                "the data waits for the free thread"
            );
            go.send(()).expect("the free thread waits");
            wait_for_free(&text);
        })
        .join()
        .expect("the test thread ends");
    }

    // followups32 (followups30 item 3 (a)): a pin that a `FileRef` guard of
    // this thread shares does not unwrap in a pin release (`Rc::try_unwrap`
    // fails): the pin drops there, and the guard keeps the version, which
    // stays live and in the registry. When the guard drops, the version
    // dies there, and its data is freed with it, not on the free thread. It
    // publishes, so no other test may build or publish stores while it runs
    // (the runner uses one thread).
    #[test]
    fn a_pin_release_keeps_a_version_that_a_guard_shares() {
        std::thread::spawn(|| {
            let (version, text) = publish_pinned("/fv/guard.ts", &["a"]);
            let file = version.file();
            let probe = file_version_probe(node(file)).expect("a live version");
            let guard = pinned_file_version(file).expect("the pin of this thread");
            drop(version);
            let go = block_free_thread();
            FREE_IN_BACKGROUND.with(|min_nodes| min_nodes.set(Some(0)));
            let dead = dead_file_versions();
            release_file_version_pins_later();
            assert!(!probe.is_freed(), "the guard keeps the version");
            assert!(lock(&VERSIONS).contains_key(&file), "a live version");
            assert!(dead_files_since(dead).0.is_empty(), "no version died");
            assert_eq!(guard.file(), file);
            drop(guard);
            assert!(probe.is_freed(), "the version dies with the guard");
            assert!(
                text.upgrade().is_none(),
                "the data is freed with the version, not on the free thread"
            );
            go.send(()).expect("the free thread waits");
        })
        .join()
        .expect("the test thread ends");
    }

    /// Starts the build's free thread (`build_task::drop_in_background`)
    /// when it has not started and blocks it, as `block_free_thread`.
    fn block_build_free_thread() -> std::sync::mpsc::Sender<()> {
        let (go, wait) = std::sync::mpsc::channel();
        crate::execute::build::build_task::drop_in_background(Blocker(wait));
        go
    }

    /// Waits up to 60 s for a free thread to free the data of `text`.
    fn wait_for_free(text: &Weak<str>) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        while text.upgrade().is_some() {
            assert!(
                std::time::Instant::now() < deadline,
                "the free thread frees the data"
            );
            std::thread::yield_now();
        }
    }

    // followups32 item 6: `tsc -b -w` frees the kept parses that a config
    // change drops with `take_data_of_dying_versions`
    // (`BuildHost::drop_kept_parses_whose_module_indicator_options_change`):
    // a version whose last holder that is dies there, before its data goes
    // to the build's free thread, so its id is dead, it is not in the
    // registry and a read of it panics while the free thread is busy. The
    // free thread frees its data later. A version with another holder lives
    // until that holder lets go. It publishes, so no other test may build or
    // publish stores while it runs (the runner uses one thread).
    #[test]
    fn a_dropped_kept_parse_dies_before_its_data_goes_to_the_free_thread() {
        std::thread::spawn(|| {
            let (version, text) = publish_pinned("/fv/kept-parse.ts", &["a"]);
            let (shared, shared_text) = publish_pinned("/fv/kept-shared.ts", &["a"]);
            let file = version.file();
            let probes = [file, shared.file()]
                .map(|file| file_version_probe(node(file)).expect("a live version"));
            // The pins of this thread go, so `version` is its last holder.
            release_file_version_pins();
            let go = block_build_free_thread();
            let dead = dead_file_versions();
            let data = take_data_of_dying_versions(vec![version, shared.clone()]);
            assert_eq!(data.len(), 1, "only a version with no other holder dies");
            crate::execute::build::build_task::drop_in_background(data);
            assert!(probes[0].is_freed(), "the version dies before the send");
            assert!(
                !lock(&VERSIONS).contains_key(&file),
                "the registry has no entry of the dead version"
            );
            assert_eq!(dead_files_since(dead).0, [file]);
            let read = std::panic::catch_unwind(|| pinned_file_version(file).is_some());
            let message = read.expect_err("a read of a dead version panics");
            assert_eq!(
                message.downcast_ref::<String>(),
                Some(&format!("file version {file} is released"))
            );
            assert!(
                text.upgrade().is_some(),
                "the data waits for the free thread"
            );
            assert!(!probes[1].is_freed(), "its other holder keeps a version");
            drop(shared);
            assert!(probes[1].is_freed(), "the last holder frees the version");
            assert!(
                shared_text.upgrade().is_none(),
                "the last holder frees the data"
            );
            go.send(()).expect("the free thread waits");
            wait_for_free(&text);
        })
        .join()
        .expect("the test thread ends");
    }

    // A node of a dead version whose id the map forgot gets no new id: the
    // id read panics (skeptic tripwire 1).
    #[test]
    #[should_panic(expected = "file version 4194301 is released")]
    fn node_id_of_a_dead_file_version_panics() {
        const DYING: usize = (1 << 22) - 3;
        const OTHER: usize = DYING - 1;
        let version = FileVersion::new(DYING);
        let id = crate::ast::get_node_id(node(DYING));
        drop(version);
        // Before the next write the map still has the id.
        assert_eq!(crate::ast::get_node_id(node(DYING)), id);
        // This write forgets the ids of the dead version.
        crate::ast::get_node_id(node(OTHER));
        crate::ast::get_node_id(node(DYING));
    }

    // A reader that finds a registry entry that does not upgrade (the
    // strong count is 0, but `Drop` has not put the id in `DEAD_FILES`
    // yet) panics, and does not read the id as no version (skeptic
    // tripwire 2).
    #[test]
    #[should_panic(expected = "file version 4194299 is released")]
    fn a_dying_file_version_is_released() {
        const DYING: usize = (1 << 22) - 5;
        // The state between the last strong drop and `FileVersion::drop`.
        lock(&VERSIONS).insert(DYING, Weak::new());
        assert!(!is_dead_file(DYING));
        with_file_version(DYING, |_| ());
    }
}
