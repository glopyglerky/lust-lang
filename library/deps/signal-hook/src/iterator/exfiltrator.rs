// `signal_hook::iterator::exfiltrator`: the sealed trait and the one
// exfiltrator this adapter has. Upstream's sealed trait carries the
// storage hooks (`Storage`, `Output`, `load`, ...); they are not here,
// so code that names them rejects. `WithRawSiginfo` is not here.

mod sealed {
    use std::fmt::Debug;

    /// Upstream's sealed half (an `unsafe trait` there; user code
    /// cannot implement either form).
    pub trait Exfiltrator: Debug + Send + Sync + 'static {}
}

/// The public, sealed exfiltrator trait (blanket over the sealed one).
pub trait Exfiltrator: sealed::Exfiltrator {}

impl<E: sealed::Exfiltrator> Exfiltrator for E {}

/// An exfiltrator providing just the signal numbers.
#[derive(Clone, Copy, Debug, Default)]
pub struct SignalOnly;

impl sealed::Exfiltrator for SignalOnly {}
