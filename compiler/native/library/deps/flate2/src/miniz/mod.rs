// The deflate half of miniz_oxide 0.9.1 (MIT/Zlib/Apache-2.0), the
// backend flate2 1.1.10's `rust_backend` compresses with. The files
// under `deflate/` and `shared.rs` are upstream's source with only
// these edits: crate paths point here (`crate::miniz::`), `alloc::`
// is `std::`, the `#[cfg(test)]` modules are cut, and Adler-32 comes
// from this adapter (`shared.rs`). This file keeps the items of
// upstream's lib.rs that the deflate half names. Keep it a copy: the
// gzip bytes `write::GzEncoder` emits must equal upstream's for every
// input, level, and write pattern.

pub mod deflate;
mod shared;

pub use self::shared::MZ_DEFAULT_WINDOW_BITS;

/// A list of flush types (upstream lib.rs).
#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum MZFlush {
    None = 0,
    Partial = 1,
    Sync = 2,
    Full = 3,
    Finish = 4,
    Block = 5,
}

/// A list of miniz successful status codes (upstream lib.rs).
#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum MZStatus {
    Ok = 0,
    StreamEnd = 1,
    NeedDict = 2,
}

/// A list of miniz failed status codes (upstream lib.rs).
#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum MZError {
    ErrNo = -1,
    Stream = -2,
    Data = -3,
    Mem = -4,
    Buf = -5,
    Version = -6,
    Param = -10_000,
}

/// How compressed data is wrapped (upstream lib.rs).
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum DataFormat {
    Zlib,
    ZLibIgnoreChecksum,
    Raw,
}

impl DataFormat {
    pub fn from_window_bits(window_bits: i32) -> DataFormat {
        if window_bits > 0 { DataFormat::Zlib } else { DataFormat::Raw }
    }

    pub fn to_window_bits(self) -> i32 {
        match self {
            DataFormat::Zlib | DataFormat::ZLibIgnoreChecksum => shared::MZ_DEFAULT_WINDOW_BITS,
            DataFormat::Raw => -shared::MZ_DEFAULT_WINDOW_BITS,
        }
    }
}

/// `Result` alias for all miniz status codes both successful and failed.
pub type MZResult = Result<MZStatus, MZError>;

/// A structure containing the result of a call to the inflate or deflate streaming functions.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct StreamResult {
    pub bytes_consumed: usize,
    pub bytes_written: usize,
    pub status: MZResult,
}

impl StreamResult {
    #[inline]
    pub const fn error(error: MZError) -> StreamResult {
        StreamResult { bytes_consumed: 0, bytes_written: 0, status: Err(error) }
    }
}
