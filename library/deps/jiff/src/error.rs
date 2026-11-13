// jiff::Error (upstream error/mod.rs), with the kinds this surface can
// make: a TZif parse error, a timestamp range error, and the
// `SystemTime` conversion errors (error/signed_duration.rs and
// util/b.rs `SignedDurationSeconds`). Display, Debug and `{:#?}` are
// upstream's: the kinds keep upstream's names and derives.

use std::marker::PhantomData;
use std::sync::Arc;

use crate::jcore;
use crate::jcore::bounds::{Bounds, RawBoundsError};

/// An error that can occur in this crate.
#[derive(Clone)]
pub struct Error {
    inner: Option<Arc<ErrorInner>>,
    // Upstream's error kinds include an `io::Error`, so upstream's Error
    // is neither UnwindSafe nor RefUnwindSafe; this keeps the same auto
    // traits.
    _auto: PhantomData<std::io::Error>,
}

#[derive(Debug)]
struct ErrorInner {
    kind: ErrorKind,
    cause: Option<Error>,
}

/// The kinds of upstream's `ErrorKind` this surface reaches, in
/// upstream's names (the derived Debug prints them).
#[derive(Debug)]
enum ErrorKind {
    Bounds(BoundsError),
    JcoreTzifParse(jcore::tz::tzif::ParseError),
    JcoreRange(jcore::bounds::RangeError),
    SignedDuration(SignedDurationError),
    Unknown,
}

impl Error {
    pub(crate) fn jcore_tzif_parse(err: jcore::tz::tzif::ParseError) -> Error {
        Error::from(ErrorKind::JcoreTzifParse(err))
    }

    pub(crate) fn jcore_range(err: jcore::bounds::RangeError) -> Error {
        Error::from(ErrorKind::JcoreRange(err))
    }

    /// Upstream `b::SignedDurationSeconds::error()` as an `Error`.
    pub(crate) fn bounds_signed_duration_seconds() -> Error {
        Error::from(ErrorKind::Bounds(BoundsError::SignedDurationSeconds(RawBoundsError::new())))
    }

    /// Upstream `context_impl`: `consequent` caused by `self`.
    fn context_impl(self, consequent: Error) -> Error {
        let mut err = consequent;
        if err.inner.is_none() {
            err = Error::from(ErrorKind::Unknown);
        }
        let inner = err.inner.as_mut().unwrap();
        assert!(inner.cause.is_none(), "cause of consequence must be `None`");
        // OK because we just created this error so the Arc
        // has one reference.
        Arc::get_mut(inner).unwrap().cause = Some(self);
        err
    }

}

impl std::error::Error for Error {}

impl core::fmt::Display for Error {
    /// Upstream prints each error of `chain()` (this error, then each
    /// cause) joined by `": "`; an error with no inner value is the
    /// `Unknown` kind and has no cause.
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut err = self;
        loop {
            let Some(ref inner) = err.inner else {
                return core::fmt::Display::fmt(&ErrorKind::Unknown, f);
            };
            core::fmt::Display::fmt(&inner.kind, f)?;
            match inner.cause {
                Some(ref cause) => {
                    f.write_str(": ")?;
                    err = cause;
                }
                None => return Ok(()),
            }
        }
    }
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if !f.alternate() {
            core::fmt::Display::fmt(self, f)
        } else {
            let Some(ref inner) = self.inner else {
                return f.debug_struct("Error").field("kind", &"None").finish();
            };
            f.debug_struct("Error").field("kind", &inner.kind).field("cause", &inner.cause).finish()
        }
    }
}

impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        use self::ErrorKind::*;

        match *self {
            Bounds(ref msg) => msg.fmt(f),
            JcoreTzifParse(ref msg) => msg.fmt(f),
            JcoreRange(ref msg) => msg.fmt(f),
            SignedDuration(ref err) => err.fmt(f),
            Unknown => f.write_str("unknown Jiff error"),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error { inner: Some(Arc::new(ErrorInner { kind, cause: None })), _auto: PhantomData }
    }
}

/// Upstream `IntoError`: a value that converts into an `Error`.
pub(crate) trait IntoError {
    fn into_error(self) -> Error;
}

impl IntoError for Error {
    fn into_error(self) -> Error {
        self
    }
}

/// Upstream `ErrorContext` (the `context` half).
pub(crate) trait ErrorContext<T, E> {
    fn context(self, consequent: impl IntoError) -> Result<T, Error>;
}

impl<T, E> ErrorContext<T, E> for Result<T, E>
where
    E: IntoError,
{
    fn context(self, consequent: impl IntoError) -> Result<T, Error> {
        self.map_err(|err| err.into_error().context_impl(consequent.into_error()))
    }
}

/// Upstream error/signed_duration.rs `Error`, with the one variant this
/// surface makes.
#[derive(Clone, Debug)]
pub(crate) enum SignedDurationError {
    ConvertSystemTime,
}

pub(crate) use self::SignedDurationError::ConvertSystemTime;

impl IntoError for SignedDurationError {
    fn into_error(self) -> Error {
        Error::from(ErrorKind::SignedDuration(self))
    }
}

impl core::fmt::Display for SignedDurationError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            SignedDurationError::ConvertSystemTime => f.write_str(
                "failed to get duration between \
                 `std::time::SystemTime` values",
            ),
        }
    }
}

/// Upstream util/b.rs `BoundsError`, with the one boundary type this
/// surface checks.
#[derive(Clone, Debug)]
enum BoundsError {
    SignedDurationSeconds(RawBoundsError<SignedDurationSeconds>),
}

impl core::fmt::Display for BoundsError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            BoundsError::SignedDurationSeconds(ref err) => err.fmt(f),
        }
    }
}

/// Upstream util/b.rs boundary type `SignedDurationSeconds`.
#[derive(Clone, Copy, Eq, PartialEq)]
struct SignedDurationSeconds(());

impl Bounds for SignedDurationSeconds {
    const WHAT: &'static str = "signed duration seconds";
    const MIN: Self::Primitive = i64::MIN;
    const MAX: Self::Primitive = i64::MAX;
    type Primitive = i64;
    type Error = BoundsError;

    fn error() -> BoundsError {
        BoundsError::SignedDurationSeconds(RawBoundsError::new())
    }
}
