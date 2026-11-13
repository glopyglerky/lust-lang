// `std::backtrace` (tf-p6 R16; orchestrator decision 10:22): only the
// forms whose output lust can give exactly. `Backtrace::disabled()`,
// `status()`, `BacktraceStatus`, and the Display (`disabled backtrace`)
// and Debug (`<disabled>`) of a disabled backtrace are std 1.99's.
// `capture()` and `force_capture()` are not here: a captured backtrace
// prints rustc's frame symbols, and `capture()` decides by the
// RUST_LIB_BACKTRACE / RUST_BACKTRACE variables at run time, so a use of
// either is a compile reject.

use core::fmt;

/// A backtrace of an OS thread. Only the disabled form is built here
/// (layout queries on it reject, as for every std source type).
pub struct Backtrace {
    inner: Inner,
}

enum Inner {
    Unsupported,
    Disabled,
}

/// The current status of a backtrace. std marks it `#[non_exhaustive]`;
/// the never-built `__` variant gives user matches the same need for a
/// wildcard arm (as `io::ErrorKind`).
#[derive(Debug, PartialEq, Eq)]
pub enum BacktraceStatus {
    /// Capturing a backtrace is not supported on this platform.
    Unsupported,
    /// Capturing was disabled through `RUST_LIB_BACKTRACE` or `RUST_BACKTRACE`.
    Disabled,
    /// A backtrace was captured.
    Captured,
    __NonExhaustive,
}

impl Backtrace {
    /// A backtrace that holds no frames.
    pub const fn disabled() -> Backtrace {
        Backtrace { inner: Inner::Disabled }
    }

    /// Whether this backtrace holds frames.
    pub fn status(&self) -> BacktraceStatus {
        match self.inner {
            Inner::Unsupported => BacktraceStatus::Unsupported,
            Inner::Disabled => BacktraceStatus::Disabled,
        }
    }
}

impl fmt::Debug for Backtrace {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner {
            Inner::Unsupported => fmt.write_str("<unsupported>"),
            Inner::Disabled => fmt.write_str("<disabled>"),
        }
    }
}

impl fmt::Display for Backtrace {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner {
            Inner::Unsupported => fmt.write_str("unsupported backtrace"),
            Inner::Disabled => fmt.write_str("disabled backtrace"),
        }
    }
}
