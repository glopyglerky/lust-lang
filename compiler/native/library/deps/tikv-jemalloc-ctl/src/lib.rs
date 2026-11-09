// Lust adapter for tikv-jemalloc-ctl 0.7.0 (docs/ARCH.md section
// 3.6): only the items whose output is the same as the real
// crate's in every program lust accepts (rule zero; BRIEF.md
// "Adapters and shims must reject what they cannot match").
//
// Kept, and why each one matches (see README.md for the audit):
// - `Name`, `AsName` (for `[u8]` and `str`), and `Error`: pure
//   values. The panic messages, the `value_type_str` key list,
//   and the error texts are upstream's, copied from its source.
// - `epoch`: a fresh real jemalloc process reads 1 first and
//   each advance, write, or update adds exactly one, with update
//   returning the new value (probed against the real crate on
//   boat, repeated runs and with `_RJEM_MALLOC_CONF` set). No
//   other item in this adapter can move the counter, and the
//   items that could move the real one are absent below.
// - `version`: the exact string of tikv-jemalloc-sys 0.7.1, the
//   sys crate the pinned 0.7.0 ctl resolves to (the driver
//   checks the adapter version against the resolved package).
//
// Removed in w-cinterop3 because their values depend on the
// jemalloc build, the machine, or `_RJEM_MALLOC_CONF`, so no
// adapter over a different allocator can return them exactly:
// every `Access<T> for Name` impl (the by-name keys
// `thread.arena`, `opt.narenas`, `thread.tcache.enabled`, the
// string keys other than the typed `version` below, and every
// `stats.*` / `arenas.*` count). A real build on a 16-core box
// reads `opt.narenas` 64, a spawned thread's `thread.arena` 1,
// and `thread.tcache.enabled` false under `tcache:false`; the
// one-arena model read 1, 0, and true. The `Access` trait stays
// declared (upstream has it) with no impls for `Name`, so a
// program that reads a key by name rejects at compile time. That
// includes ts-rust's `program.rs` thread-arena form and
// `jemalloc_layout.rs`: their output happens not to depend on
// the values, but the same API in another program prints them,
// so the API cannot stay. The typed key modules other than
// `epoch` and `version`, `Mib`, and `MibStr` are absent for the
// same reason. The `stats` and `profiling` features are not
// implemented.

use core::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

const EPERM: i32 = 1;
const ENOENT: i32 = 2;
const EAGAIN: i32 = 11;
const EFAULT: i32 = 14;
const EINVAL: i32 = 22;

/// Errors of the mallctl family: a non-zero errno code, as upstream.
#[derive(Copy, Clone, PartialEq)]
pub struct Error(i32);

/// Result type, as upstream.
pub type Result<T> = core::result::Result<T, Error>;

/// The error text upstream prints for `code`.
fn description(code: i32) -> Option<&'static str> {
    match code {
        EINVAL => Some(
            "`newp` is not `NULL`, and `newlen` is too large or too small. Alternatively, `*oldlenp` is too large or too small; in this case as much data as possible are read despite the error.",
        ),
        ENOENT => Some("`name` or `mib` specifies an unknown/invalid value."),
        EPERM => Some("Attempt to read or write `void` value, or attempt to write read-only value."),
        EAGAIN => Some("A memory allocation failure occurred."),
        EFAULT => Some("An interface with side effects failed in some way not directly related to `mallctl*()` read/write processing."),
        _ => None,
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match description(self.0) {
            Some(m) => write!(f, "{m}"),
            None => write!(f, "Unknown error code: \"{}\".", self.0),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

#[cfg(feature = "use_std")]
impl std::error::Error for Error {}

/// A name in the mallctl namespace: an unsized wrapper over the
/// null-terminated bytes, as upstream.
#[repr(transparent)]
#[derive(PartialEq, Eq)]
pub struct Name([u8]);

/// Converts a null-terminated byte-string into a [`Name`].
pub trait AsName {
    /// Converts a null-terminated byte-string into a [`Name`].
    fn name(&self) -> &Name;
}

impl AsName for [u8] {
    fn name(&self) -> &Name {
        assert!(!self.is_empty(), "cannot create Name from empty byte-string");
        assert_eq!(
            *self.last().unwrap(),
            b'\0',
            "cannot create Name from non-null-terminated byte-string \"{}\"",
            core::str::from_utf8(self).unwrap()
        );
        self
    }
}

impl AsName for str {
    fn name(&self) -> &Name {
        self.as_bytes().name()
    }
}

/// True when `v` starts with `pre` (byte slices have no
/// `starts_with` in the lust subset yet).
fn starts_with(v: &[u8], pre: &[u8]) -> bool {
    v.len() >= pre.len() && &v[..pre.len()] == pre
}

/// True when `v` ends with `suf`.
fn ends_with(v: &[u8], suf: &[u8]) -> bool {
    v.len() >= suf.len() && &v[v.len() - suf.len()..] == suf
}

impl Name {
    /// The key bytes without the null terminator.
    fn key(&self) -> &[u8] {
        &self.0[..self.0.len() - 1]
    }

    /// True when this key names a string value, as upstream's list.
    pub fn value_type_str(&self) -> bool {
        let name = self.key();
        if name.is_empty() {
            return false;
        }
        match name {
            b"version"
            | b"config.malloc_conf"
            | b"opt.metadata_thp"
            | b"opt.dss"
            | b"opt.percpu_arena"
            | b"opt.stats_print_opts"
            | b"opt.junk"
            | b"opt.thp"
            | b"opt.prof_prefix"
            | b"thread.prof.name"
            | b"prof.dump" => true,
            v if starts_with(v, b"arena.") && ends_with(v, b".dss") => true,
            v if starts_with(v, b"stats.arenas.") && ends_with(v, b".dss") => true,
            _ => false,
        }
    }

    /// The name as a null-terminated byte-string.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", core::str::from_utf8(&self.0).unwrap())
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", core::str::from_utf8(&self.0).unwrap())
    }
}

/// Safe read access to the mallctl namespace, as upstream
/// declares it. This adapter has no impls of it for `Name`:
/// every by-name key's value depends on the jemalloc build or
/// the machine (see the file comment), so a call through this
/// trait on a `Name` is a compile-time reject in lust, never an
/// answer with a different value.
pub trait Access<T> {
    /// Read the key at `self`.
    fn read(&self) -> Result<T>;
    /// Write `value` at the key `self`.
    fn write(&self, value: T) -> Result<()>;
    /// Write `value` at the key `self` returning its previous value.
    fn update(&self, value: T) -> Result<T>;
}

/// The epoch counter. Real jemalloc starts at 1 and each advance
/// adds one; nothing else in this adapter can move it, so the
/// counts a program prints are the real crate's counts.
static EPOCH: AtomicU64 = AtomicU64::new(1);

/// The `epoch` key: many statistics are cached and refresh when
/// the epoch advances. Here it only counts, as a fresh real
/// process observes it (first read 1, each advance adds one).
#[allow(non_camel_case_types)]
pub struct epoch;

/// Upstream's `epoch` is a unit struct whose key fns are associated
/// fns (so `use epoch::advance` rejects, as upstream).
impl epoch {
    /// Reads the epoch.
    pub fn read() -> Result<u64> {
        Ok(EPOCH.load(Ordering::SeqCst))
    }

    /// Advances the epoch, ignoring `value`, as upstream.
    pub fn write(_value: u64) -> Result<()> {
        EPOCH.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    /// Advances the epoch and returns its new value, as
    /// upstream's docs state for this key.
    pub fn update(_value: u64) -> Result<u64> {
        Ok(EPOCH.fetch_add(1, Ordering::SeqCst) + 1)
    }

    /// Advances the epoch returning its latest value.
    pub fn advance() -> Result<u64> {
        epoch::update(1)
    }
}

/// The jemalloc version string of tikv-jemalloc-sys 0.7.1, with
/// its trailing null, as `version` reads it upstream.
const VERSION: &str = "5.3.1-0-g81034ce1f1373e37dc865038e1bc8eeecf559ce8\0";

/// The `version` key: the jemalloc version string (a unit struct
/// with an associated `read`, as upstream).
#[allow(non_camel_case_types)]
pub struct version;

impl version {
    /// Reads the version string (with its trailing null, as
    /// upstream returns it).
    pub fn read() -> Result<&'static str> {
        Ok(VERSION)
    }
}
