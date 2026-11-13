//! Go: internal/bundled/embed.go and embed_generated.go (the `!noembed`
//! build, the default).
//!
//! PORT: the pinned reference build embeds the libs and names them
//! `bundled:///libs/lib.*.d.ts` (see the oracle `*.files.txt` lists). The
//! lib texts come from `crates/ts_goport/libs`, a copy of the pinned
//! `internal/bundled/libs` (see its PROVENANCE.md).
//!
//! PORT: the wasm build embeds the libs packed (one LZMA stream, 0.31 MB in
//! place of 3.79 MB; see `parts/goport_util/build.rs`) and unpacks the
//! stream up to each lib on its first read. The names and texts do not
//! change.

#[cfg(not(target_family = "wasm"))]
use super::{SNAPSHOT_TEXT_MIN, };
use crate::frontend::prelude::*;
#[cfg(not(target_family = "wasm"))]
use std::sync::OnceLock;
use std::time::SystemTime;

// Go: embed.go:13 embedded
// PORT: renamed; the Go names `Embedded` and `embedded` share a snake name.
pub(super) const EMBEDDED_UNEXPORTED: bool = true;

// Go: embed.go:15 scheme
const SCHEME: &str = "bundled:///";

// Go: embed.go:17 splitPath
fn split_path(path: &str) -> Option<&str> {
    path.strip_prefix(SCHEME)
}

// Go: embed.go:21 libPath
pub fn lib_path() -> String {
    format!("{SCHEME}libs")
}


// Go: embed.go:25 IsBundled
pub fn is_bundled(path: &str) -> bool {
    split_path(path).is_some()
}


/// wasm: always `None`. The wasm build reads no snapshot
/// (`bundled_lib_name`), so no key needs the hash.
// PORT: not in Go.
#[cfg(target_family = "wasm")]
#[must_use]
pub fn embedded_text_hash(_text: &str) -> Option<u64> {
    None
}

/// The base name of bundled lib path `path` (`bundled:///libs/lib.dom.d.ts`
/// gives `lib.dom.d.ts`). `None` for any other path. The lib parse and bind
/// snapshots find a lib file by this name.
///
/// wasm: always `None`, so no lib loads a snapshot and the link drops the
/// snapshot blobs (3 MB) and their decoders. On wasm a snapshot load took
/// as long as the live parse and bind of the lib.
// PORT: not in Go.
#[must_use]
pub fn bundled_lib_name(path: &str) -> Option<&str> {
    if cfg!(target_family = "wasm") {
        return None;
    }
    split_path(path)?.strip_prefix("libs/")
}

// wrappedFS is implemented directly rather than going through [io/fs.FS].
// Our vfs.FS works with file contents in terms of strings, and that's
// what go:embed does under the hood, but going through fs.FS will cause
// copying to []byte and back.

// Go: embed.go:35 wrappedFS
struct WrappedFs {
    fs: Rc<dyn Fs>,
}


// Go: embed.go:41 wrapFS
// PERF: the wrapper of the OS file system is one value per thread, so
// `is_wrapped_os_fs` can know it (see there).



// Go: embed.go:133 fileInfo
// PORT: the Go `fileInfo` type is the shared `FileInfo` value. Its
// `ModTime` is the Go zero time (`None`) and `Info()` returns itself
// (`DirEntryInfo::Known`).
fn new_file_info(name: &str, mode: FileMode, size: i64) -> FileInfo {
    FileInfo {
        name: name.to_string(),
        size,
        mode,
        mod_time: None,
    }
}

// Go: embed_generated.go:238 embeddedContents
// PORT: the Go map is built once from `EMBEDDED_CONTENTS`.

// Go: embed_generated.go:238 embeddedContents
// PORT: wasm unpacks the one stream of build.rs (`libs.lzma`, the libs in
// `PACKED_LIBS` order) up to lib `rest`, and keeps each text it unpacks for
// the life of the process. The reader stays too, with its 3.8 MB window
// (one stream per lib needed at most 2.35 MB, for lib.dom.d.ts, at a time).
#[cfg(target_family = "wasm")]
fn embedded_contents(rest: &str) -> Option<&'static str> {
    use std::io::Read;
    use std::sync::{Mutex, PoisonError};
    struct Unpacked {
        reader: lzma_rust2::LzmaReader<&'static [u8]>,
        texts: Vec<&'static str>,
    }
    static UNPACKED: Mutex<Option<Unpacked>> = Mutex::new(None);
    let index = packed_lib_index(rest)?;
    let mut unpacked = UNPACKED.lock().unwrap_or_else(PoisonError::into_inner);
    let unpacked = unpacked.get_or_insert_with(|| {
        let packed: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/libs.lzma"));
        Unpacked {
            reader: lzma_rust2::LzmaReader::new_mem_limit(packed, u32::MAX, None)
                .expect("build.rs packs the libs"),
            texts: Vec::new(),
        }
    });
    while unpacked.texts.len() <= index {
        let mut text = vec![0; PACKED_LIBS[unpacked.texts.len()].1];
        unpacked
            .reader
            .read_exact(&mut text)
            .expect("build.rs packs the libs");
        let text = String::from_utf8(text).expect("a lib is UTF-8");
        unpacked.texts.push(Box::leak(text.into_boxed_str()));
    }
    Some(unpacked.texts[index])
}


/// wasm: the size of lib `rest`, which unpacks nothing.
#[cfg(target_family = "wasm")]
fn embedded_len(rest: &str) -> Option<usize> {
    packed_lib_index(rest).map(|index| PACKED_LIBS[index].1)
}

/// wasm: the position of lib `rest` in `PACKED_LIBS`, when it is one of the
/// bundled libs (`EMBEDDED_CONTENTS`).
#[cfg(target_family = "wasm")]
fn packed_lib_index(rest: &str) -> Option<usize> {
    if !EMBEDDED_CONTENTS.contains(&rest) {
        return None;
    }
    let index = PACKED_LIBS.iter().position(|&(path, _)| path == rest);
    Some(index.expect("build.rs packs each bundled lib"))
}

// wasm: `PACKED_LIBS`, the path and size of each lib in `libs.lzma` order.
#[cfg(target_family = "wasm")]
include!(concat!(env!("OUT_DIR"), "/libs.rs"));

/// wasm: the text that `parts/goport_util/build.rs` packed (an `.lzma`
/// stream): the diagnostic message texts.
#[cfg(target_family = "wasm")]
pub(crate) fn unpack(packed: &[u8]) -> String {
    use std::io::Read;
    let mut text = Vec::new();
    lzma_rust2::LzmaReader::new_mem_limit(packed, u32::MAX, None)
        .and_then(|mut reader| reader.read_to_end(&mut text))
        .expect("build.rs packs the text");
    String::from_utf8(text).expect("a packed text is UTF-8")
}

/// One `EMBEDDED_CONTENTS` entry (see `bundled_lib!`).
#[cfg(not(target_family = "wasm"))]
type EmbeddedLib = (&'static str, &'static str, u64);
/// wasm: the path. The texts are in one stream (`embedded_contents`).
#[cfg(target_family = "wasm")]
type EmbeddedLib = &'static str;

// PORT: one `EMBEDDED_CONTENTS` entry, from `crates/ts_goport/libs`: the
// path, the text and its compile-time `snapshot_text_hash`. The include path
// is relative to this file: this file builds in `goport_util`, whose
// manifest dir is not `crates/ts_goport`.
// `crates/ts_goport/scripts/copy-libs.sh` copies the same set for a noembed
// build.
#[cfg(not(target_family = "wasm"))]
macro_rules! bundled_lib {
    ($name:literal) => {{
        const TEXT: &str = include_str!(concat!("../../../libs/", $name));
        // One compile-time evaluation per lib, so each stays under the
        // rustc step limit.
        const HASH: u64 = snapshot_text_hash(TEXT);
        (concat!("libs/", $name), TEXT, HASH)
    }};
}

/// wasm: one `EMBEDDED_CONTENTS` entry: the path.
// PORT: not in Go.
#[cfg(target_family = "wasm")]
macro_rules! bundled_lib {
    ($name:literal) => {
        concat!("libs/", $name)
    };
}


// Go: embed_generated.go:9 (the go:embed variables)

#[cfg(test)]
mod tests {
    use super::*;

    /// A snapshot lib has its compile-time hash only as the embedded text
    /// itself: a copy of the text, or a small lib, has none.
    #[test]
    fn embedded_text_hash_is_compile_time_hash() {
        let dom = bundled_text("bundled:///libs/lib.dom.d.ts").expect("lib.dom");
        assert!(dom.len() >= SNAPSHOT_TEXT_MIN);
        assert_eq!(embedded_text_hash(dom), Some(const_hash(dom.as_bytes())));
        assert_eq!(embedded_text_hash(&dom.to_string()), None);
        let small = bundled_text("bundled:///libs/lib.es2015.d.ts").expect("lib.es2015");
        assert_eq!(embedded_text_hash(small), None);
    }
}
