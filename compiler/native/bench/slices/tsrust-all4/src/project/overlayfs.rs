//! Go `internal/project/overlayfs.go`.
//!
//! PORT: one thread (project/dirty/interfaces.rs). Go `sync.Once` + value is
//! a `OnceCell`; `mu` is dropped. Go `xxh3.Uint128` is `u128`. A Go
//! `FileHandle` is `Rc<dyn FileHandle>` (nil is `None`): an `*Overlay` is
//! `Rc<Overlay>` and a `*cachedFile` is `Rc<RefCell<CachedFile>>` (the dirty maps
//! change it after sharing). Go `string` results are owned `String`s, except
//! the file text: a file keeps it as an `Arc<str>`, which a Go string is (an
//! immutable shared value), so a reader that only reads it takes
//! `shared_content` and does not copy it.

use crate::project::prelude::*;

use crate::frontend::core_textchange::TextChange;
use std::cell::{Cell, OnceCell};
use std::sync::Arc;
use crate::xxhash_rust::xxh3::xxh3_128;

// Go: project/overlayfs.go:22 FileContent
pub trait FileContent {
    /// Go `Content`: an owned copy of the text.
    fn content(&self) -> String {
        self.shared_content().to_string()
    }
    /// Go `Content` without the copy: the text that the file holds. Use it
    /// where the text is read per result (a language service reads a file
    /// once per location it converts).
    fn shared_content(&self) -> Arc<str>;
    fn hash(&self) -> u128;
}

// Go: project/overlayfs.go:27 FileHandle
pub trait FileHandle: FileContent {
    fn file_name(&self) -> String;
    fn version(&self) -> i32;
    fn matches_disk_text(&self) -> bool;
    fn is_overlay(&self) -> bool;
    fn lsp_line_map(&self) -> Rc<lsconv::LSPLineMap>;
    fn ecma_line_info(&self) -> Rc<sourcemap::lineinfo::ECMALineInfo>;
    fn kind(&self) -> ScriptKind;
}

// Go: project/overlayfs.go:38 fileBase
// PORT: `hash` is a `Cell` because Go writes it on an overlay that may be
// shared (processChanges). `content` is an `Arc<str>`: a Go string is shared,
// and a clone of the file (Go `Clone`, a new overlay with the same text)
// shares it too.
#[derive(Debug, Default)]
pub struct FileBase {
    pub file_name: String,
    pub content: Arc<str>,
    pub hash: Cell<u128>,

    pub line_map: OnceCell<Rc<lsconv::LSPLineMap>>,
    pub line_info: OnceCell<Rc<sourcemap::lineinfo::ECMALineInfo>>,
}

impl FileBase {
    // Go: project/overlayfs.go:49 fileBase.FileName
    pub fn file_name(&self) -> String {
        self.file_name.clone()
    }

    // Go: project/overlayfs.go:53 fileBase.Hash
    pub fn hash(&self) -> u128 {
        self.hash.get()
    }

    // Go: project/overlayfs.go:57 fileBase.Content
    pub fn content(&self) -> String {
        self.content.to_string()
    }

    /// Go `fileBase.Content` without the copy (`FileContent::shared_content`).
    pub fn shared_content(&self) -> Arc<str> {
        Arc::clone(&self.content)
    }

    // Go: project/overlayfs.go:61 fileBase.LSPLineMap

    // Go: project/overlayfs.go:68 fileBase.ECMALineInfo
    pub fn ecma_line_info(&self) -> Rc<sourcemap::lineinfo::ECMALineInfo> {
        self.line_info
            .get_or_init(|| {
                let line_starts = compute_ecma_line_starts(&self.content);
                Rc::new(sourcemap::lineinfo::create_ecma_line_info(
                    &self.content,
                    line_starts,
                ))
            })
            .clone()
    }
}

// Go: project/overlayfs.go:76 cachedFile (ts#64291: was diskFile)
// PORT: Go embeds `fileBase`; here it is the field `file_base`.
#[derive(Debug, Default)]
pub struct CachedFile {
    pub file_base: FileBase,
    pub needs_reload: bool,
    pub realpath_path: tspath::Path,
}

// Go: project/overlayfs.go:82 newCachedFile
// PORT: the file keeps `content`. An `Arc<str>` is shared, a `String` or
// `&str` is copied once.
pub fn new_cached_file(file_name: &str, content: impl Into<Arc<str>>) -> Rc<RefCell<CachedFile>> {
    let content: Arc<str> = content.into();
    let hash = xxh3_128(content.as_bytes());
    Rc::new(RefCell::new(CachedFile {
        file_base: FileBase {
            file_name: file_name.to_string(),
            content,
            hash: Cell::new(hash),
            ..FileBase::default()
        },
        ..CachedFile::default()
    }))
}

// Go: project/overlayfs.go:90 NewCachedFileHandle (ts#64291)

impl CachedFile {
    // Go: project/overlayfs.go:96 cachedFile.Version
    pub fn version(&self) -> i32 {
        0
    }

    // Go: project/overlayfs.go:100 cachedFile.MatchesDiskText
    pub fn matches_disk_text(&self) -> bool {
        !self.needs_reload
    }

    // Go: project/overlayfs.go:104 cachedFile.IsOverlay
    pub fn is_overlay(&self) -> bool {
        false
    }

    // Go: project/overlayfs.go:108 cachedFile.Kind
    // PORT: tsgo #4712 reverts #4628 here. An extensionless file keeps
    // `ScriptKind::UNKNOWN`; `new_parse_cache_key` picks TS for the parse.

    // Go: project/overlayfs.go:112 cachedFile.Clone
    // PORT: Go `Clone`; `clone_` keeps it apart from `std::clone::Clone`.
    pub fn clone_(&self) -> Rc<RefCell<CachedFile>> {
        Rc::new(RefCell::new(CachedFile {
            realpath_path: self.realpath_path.clone(),
            file_base: FileBase {
                file_name: self.file_base.file_name.clone(),
                content: Arc::clone(&self.file_base.content),
                hash: Cell::new(self.file_base.hash.get()),
                ..FileBase::default()
            },
            ..CachedFile::default()
        }))
    }
}

// PORT: the dirty maps hold `*cachedFile` and call its `Clone`.
impl dirty::Cloneable for Rc<RefCell<CachedFile>> {
    fn clone_(&self) -> Self {
        self.borrow().clone_()
    }
}

// Go: project/overlayfs.go:96 `var _ FileHandle = (*cachedFile)(nil)`
// PORT: the methods of `fileBase` are promoted through the embedding.
impl FileContent for RefCell<CachedFile> {
    fn shared_content(&self) -> Arc<str> {
        self.borrow().file_base.shared_content()
    }

    fn hash(&self) -> u128 {
        self.borrow().file_base.hash()
    }
}


// Go: project/overlayfs.go:123 Overlay
// PORT: Go embeds `fileBase`; here it is the field `file_base`. `version`
// and `matches_disk_text` are `Cell`s because Go writes them on an overlay
// that may be shared (processChanges).
#[derive(Debug, Default)]
pub struct Overlay {
    pub file_base: FileBase,
    pub version: Cell<i32>,
    pub kind: ScriptKind,
    pub matches_disk_text: Cell<bool>,
}

// Go: project/overlayfs.go:130 newOverlay
// PORT: the overlay keeps `content`. An `Arc<str>` is shared, a `String` or
// `&str` is copied once.
pub fn new_overlay(
    file_name: &str,
    content: impl Into<Arc<str>>,
    version: i32,
    kind: ScriptKind,
) -> Overlay {
    let content: Arc<str> = content.into();
    let hash = xxh3_128(content.as_bytes());
    Overlay {
        file_base: FileBase {
            file_name: file_name.to_string(),
            content,
            hash: Cell::new(hash),
            ..FileBase::default()
        },
        version: Cell::new(version),
        kind,
        ..Overlay::default()
    }
}

impl Overlay {
    // Go: project/overlayfs.go:144 Overlay.Text
    pub fn text(&self) -> String {
        self.file_base.content.to_string()
    }

    // Go: project/overlayfs.go:163 Overlay.computeMatchesDiskText
    // !!! optimization: incorporate mtime
    // PORT: Go named results `(matchesDiskText bool, exists bool)`.
    pub fn compute_matches_disk_text(&self, fs: &dyn vfs::Fs) -> (bool, bool) {
        if tspath::is_dynamic_file_name(&self.file_base.file_name) {
            return (false, false);
        }
        let (disk_content, ok) = fs.read_file(&self.file_base.file_name);
        if !ok {
            return (false, false);
        }
        (
            xxh3_128(disk_content.as_bytes()) == self.file_base.hash(),
            true,
        )
    }
}

// Go: project/overlayfs.go:116 `var _ FileHandle = (*Overlay)(nil)`
// PORT: the methods of `fileBase` are promoted through the embedding.
impl FileContent for Overlay {
    fn shared_content(&self) -> Arc<str> {
        self.file_base.shared_content()
    }

    fn hash(&self) -> u128 {
        self.file_base.hash()
    }
}


// PORT: Go passes an `*Overlay` to `lsconv.FromLSPRange` as an
// `lsconv.Script` (its `FileName` and `Text` methods and the tsgo#4712
// methods below).
impl lsconv::Script for Overlay {
    fn file_name(&self) -> &str {
        &self.file_base.file_name
    }

    fn text(&self) -> lsconv::ScriptText<'_> {
        lsconv::ScriptText::Borrowed(&self.file_base.content)
    }

    // Go: project/overlayfs.go:148 Overlay.OriginalFileName (tsgo#4712)
    fn original_file_name(&self) -> &str {
        &self.file_base.file_name
    }

    // Go: project/overlayfs.go:153 Overlay.SpanMap (tsgo#4712)
    // SpanMap and OriginalText satisfy lsconv.Script. An overlay holds the editor's raw text (for a
    // content-mapped file, that is the original foreign text, not the transformed output), so it never
    // carries a span map and its original text is its own text.
    fn span_map(&self) -> Option<&crate::spanmap::SpanMap> {
        None
    }

    // Go: project/overlayfs.go:155 Overlay.OriginalText (tsgo#4712)
    fn original_text(&self) -> lsconv::ScriptText<'_> {
        lsconv::ScriptText::Borrowed(&self.file_base.content)
    }
}

// Go: project/overlayfs.go:182 overlayFS
// PORT: `mu` is dropped; `overlays` and `overlayDirectories` are replaced
// after sharing, so they are `RefCell`s. Go `map[tspath.Path]*Overlay` is an
// `IndexMap` (insertion order; PORT: Go map order is random), because the
// project collection ranges over the overlays when it picks inferred project
// roots. Go shares the map (a reference), so it is an `Rc` map here.
pub struct OverlayFS {
    pub to_path: Rc<dyn Fn(&str) -> tspath::Path>,
    pub host: Rc<dyn vfs::Fs>,
    pub position_encoding: lsproto::PositionEncodingKind,

    pub overlays: RefCell<Rc<IndexMap<tspath::Path, Rc<Overlay>>>>,
    pub overlay_directories: RefCell<Rc<OverlayDirectories>>,
}

/// Go `map[tspath.Path]map[tspath.Path]string` of `overlayFS.overlayDirectories`.
// PORT: Go ranges over a directory map (`GetAccessibleEntries`). A Go map
// of at most 8 entries is one group of 8 slots: an insert takes the first
// free slot, a delete frees its slot, and a range starts at a random slot
// and wraps. Go makes the directory maps again from `overlays` at each
// change (`createOverlayDirectories`) and never deletes from them. So the
// slots hold the children in the order of the range over `overlays`, and Go
// gives a rotation of that order. The inner map is an `IndexMap` (insertion
// order), filled in the order of the port's `overlays`. When that order is
// Go's slot order of `overlays`, the port gives the rotation that starts at
// the first slot, which Go can give. Go can differ in two cases:
// - A close and then an open: Go puts the new overlay in the free slot of
//   the closed one, but the port puts it last (`shift_remove`, then insert).
// - Above 8 entries (in `overlays` or in one directory): Go's order depends
//   on the hash seed of the map.
// An FxHashMap gave orders that Go never gives, also with no close
// (editfuzz2 R1).
pub type OverlayDirectories = FxHashMap<tspath::Path, IndexMap<tspath::Path, String>>;

// Go: project/overlayfs.go:192 LayeredFileSystem (ts#64291)
pub trait LayeredFileSystem: vfs::Fs + FileHandleSource {
    fn overlays(&self) -> Rc<IndexMap<tspath::Path, Rc<Overlay>>>;
}

// Go: project/overlayfs.go:198 RebasableFileSystem (ts#64291)
pub trait RebasableFileSystem: vfs::Fs {
    fn base_file_system(&self) -> Rc<dyn vfs::Fs>;
    fn with_base_file_system(&self, base: Rc<dyn vfs::Fs>) -> Rc<dyn LayeredFileSystem>;
}

/// PORT: the Go type assertions on a `vfs.FS` value that the file system
/// layers use (ts#64291): `fs.(FileHandleSource)`, `fs.(LayeredFileSystem)`,
/// `fs.(RebasableFileSystem)`, `fs.(FileChangeExpander)` and
/// `fs.(*overlayFS)` (and, through `as_any`, the other packages' concrete
/// layer types). `as_fs_layer` returns this view for a layer; each
/// default is a failed assertion.
pub trait FsLayer {
    fn as_file_handle_source(&self) -> Option<&dyn FileHandleSource> {
        None
    }
    fn as_layered_file_system(&self) -> Option<&dyn LayeredFileSystem> {
        None
    }
    fn as_rebasable_file_system(&self) -> Option<&dyn RebasableFileSystem> {
        None
    }
    fn as_file_change_expander(&self) -> Option<&dyn FileChangeExpander> {
        None
    }
    fn as_overlay_fs(&self) -> Option<&OverlayFS> {
        None
    }
    /// A concrete layer type (Go `fs.(*T)` in another package).
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        None
    }
}







// Go: project/overlayfs.go:204 newOverlayFS

impl OverlayFS {
    // Go: project/overlayfs.go:220 overlayFS.Overlays
    pub fn overlays(&self) -> Rc<IndexMap<tspath::Path, Rc<Overlay>>> {
        self.overlays.borrow().clone()
    }

    // Go: project/overlayfs.go:243 overlayFS.GetFile (ts#64291: was getFile)
    pub fn get_file(&self, file_name: &str) -> Option<Rc<dyn FileHandle>> {
        self.get_file_by_path(file_name, &(self.to_path)(file_name))
    }

    // Go: project/overlayfs.go:247 overlayFS.GetFileByPath
}

// Go: project/overlayfs.go:226 layerOverlayFileSystem (ts#64291)

// Go: project/overlayfs.go:218 `_ FileHandleSource = (*overlayFS)(nil)`
impl FileHandleSource for OverlayFS {
    fn get_file(&self, file_name: &str) -> Option<Rc<dyn FileHandle>> {
        OverlayFS::get_file(self, file_name)
    }

    fn get_file_by_path(&self, file_name: &str, path: &tspath::Path) -> Option<Rc<dyn FileHandle>> {
        OverlayFS::get_file_by_path(self, file_name, path)
    }
}

// Go: project/overlayfs.go:219 `_ LayeredFileSystem = (*overlayFS)(nil)`


// Go: project/overlayfs.go:217 `_ vfs.FS = (*overlayFS)(nil)`

// Go: project/overlayfs.go:362 overlayFileInfo (ts#64291)
// PORT: Go `vfs.FileInfo` is the `vfs::FileInfo` value of the Go methods:
// Name, Size, Mode 0o444, zero ModTime.

// Go: project/overlayfs.go:373 overlayDirectoryInfo (ts#64291)
// PORT: Mode is `ModeDir | 0o555`, zero ModTime.
pub fn overlay_directory_info(name: String) -> vfs::FileInfo {
    vfs::FileInfo {
        name,
        size: 0,
        mode: vfs::FileMode(vfs::FileMode::DIR.0 | 0o555),
        mod_time: None,
    }
}

// Go: project/overlayfs.go:384 createOverlayDirectories (ts#64291)
// PORT: Go ranges over the overlay map (random order); the IndexMap order
// here. It is the insertion order of each directory map
// (`OverlayDirectories`), so a directory lists its open files in the order
// they were opened.

impl OverlayFS {
    // Go: project/overlayfs.go:407 overlayFS.processChanges
    // PORT: Go takes the slice; here a borrowed slice. The per-file events
    // keep references into it where Go keeps pointers to copies. Go ranges
    // over `fileEventMap` (random order); the port keeps the order in which
    // each URI first appears.
}
