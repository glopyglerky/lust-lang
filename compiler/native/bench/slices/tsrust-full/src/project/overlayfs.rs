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

// Go: project/overlayfs.go:22 FileContent

// Go: project/overlayfs.go:27 FileHandle

// Go: project/overlayfs.go:38 fileBase
// PORT: `hash` is a `Cell` because Go writes it on an overlay that may be
// shared (processChanges). `content` is an `Arc<str>`: a Go string is shared,
// and a clone of the file (Go `Clone`, a new overlay with the same text)
// shares it too.


// Go: project/overlayfs.go:76 cachedFile (ts#64291: was diskFile)
// PORT: Go embeds `fileBase`; here it is the field `file_base`.

// Go: project/overlayfs.go:82 newCachedFile
// PORT: the file keeps `content`. An `Arc<str>` is shared, a `String` or
// `&str` is copied once.

// Go: project/overlayfs.go:90 NewCachedFileHandle (ts#64291)


// PORT: the dirty maps hold `*cachedFile` and call its `Clone`.

// Go: project/overlayfs.go:96 `var _ FileHandle = (*cachedFile)(nil)`
// PORT: the methods of `fileBase` are promoted through the embedding.


// Go: project/overlayfs.go:123 Overlay
// PORT: Go embeds `fileBase`; here it is the field `file_base`. `version`
// and `matches_disk_text` are `Cell`s because Go writes them on an overlay
// that may be shared (processChanges).

// Go: project/overlayfs.go:130 newOverlay
// PORT: the overlay keeps `content`. An `Arc<str>` is shared, a `String` or
// `&str` is copied once.


// Go: project/overlayfs.go:116 `var _ FileHandle = (*Overlay)(nil)`
// PORT: the methods of `fileBase` are promoted through the embedding.


// PORT: Go passes an `*Overlay` to `lsconv.FromLSPRange` as an
// `lsconv.Script` (its `FileName` and `Text` methods and the tsgo#4712
// methods below).

// Go: project/overlayfs.go:182 overlayFS
// PORT: `mu` is dropped; `overlays` and `overlayDirectories` are replaced
// after sharing, so they are `RefCell`s. Go `map[tspath.Path]*Overlay` is an
// `IndexMap` (insertion order; PORT: Go map order is random), because the
// project collection ranges over the overlays when it picks inferred project
// roots. Go shares the map (a reference), so it is an `Rc` map here.

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

// Go: project/overlayfs.go:198 RebasableFileSystem (ts#64291)








// Go: project/overlayfs.go:204 newOverlayFS


// Go: project/overlayfs.go:226 layerOverlayFileSystem (ts#64291)

// Go: project/overlayfs.go:218 `_ FileHandleSource = (*overlayFS)(nil)`

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

