//! Go panics and unported-code hits: `GoPanic`, `go_panic`, `unported!`
//! and the unported registry. They live in `goport_util` as the module
//! `core`, so util files keep their `crate::core::` paths and
//! `$crate::core::record_unported` resolves. `ts_goport`'s `core.rs`
//! re-exports them.

/// Unported hits of every thread, by Go name.
static UNPORTED_NAMES: std::sync::Mutex<std::collections::BTreeMap<&'static str, u64>> =
    std::sync::Mutex::new(std::collections::BTreeMap::new());




/// Marks unported Go code. It records the hit, then panics so the gap is
/// loud. Use only where a port is missing, never as a fallback.
#[macro_export]
macro_rules! unported {
    ($go_name:expr) => {{
        $crate::core::record_unported($go_name);
        panic!("unported Go code: {}", $go_name)
    }};
}

/// The panic payload of `go_panic`.
pub struct GoPanic {
    /// The Go panic value as the Go runtime prints it (port form). It is
    /// also the Go `%v` of the value that `recover()` returns.
    pub message: String,
    /// The Go type of a value whose type is a named string type, such as
    /// `lsproto.DocumentUri` (`go_panic_typed`). The runtime prints it as
    /// `<type>("<message>")`. `None` for a plain string or an error.
    pub go_type: Option<&'static str>,
    /// A Go `recover()` raised the value again with `panic(r)`
    /// (`go_repanic`). The runtime adds ` [recovered, repanicked]`.
    pub repanicked: bool,
    /// The port site, for the stderr report.
    pub location: &'static std::panic::Location<'static>,
}

/// Go `panic(message)` at a site where the pinned Go panics on the same
/// input. It is not a port gap, so the run ends as the Go runtime ends it:
/// guards that keep a run going after a port gap pass it on
/// (`resume_go_panic`), and the bins write the output so far, print it with
/// `print_go_panic` and exit `EXIT_GO_PANIC`. Other panics stay port gaps
/// (`execute::tsc::EXIT_UNPORTED`).
#[track_caller]
pub fn go_panic(message: String) -> ! {
    std::panic::panic_any(GoPanic {
        message,
        go_type: None,
        repanicked: false,
        location: std::panic::Location::caller(),
    })
}

/// `go_panic` with a value of the named Go string type `go_type`, for
/// example `panic("overlay not found: " + uri)` where `uri` is a
/// `lsproto.DocumentUri`. `recover()` gives the same text, but the runtime
/// prints `panic: <go_type>("<message>")`.
#[track_caller]
pub fn go_panic_typed(go_type: &'static str, message: String) -> ! {
    std::panic::panic_any(GoPanic {
        message,
        go_type: Some(go_type),
        repanicked: false,
        location: std::panic::Location::caller(),
    })
}

/// Go `if r := recover(); r != nil { ...; panic(r) }`: raises a caught
/// panic again. A `go_panic` value is marked, so the runtime line gets
/// ` [recovered, repanicked]`. Any other payload continues as it is.
pub fn go_repanic(mut payload: Box<dyn std::any::Any + Send>) -> ! {
    if let Some(panic) = payload.downcast_mut::<GoPanic>() {
        panic.repanicked = true;
    }
    std::panic::resume_unwind(payload)
}



thread_local! {
    /// How many `go_recover` calls this thread is inside.
    static GO_RECOVER_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}


/// True inside `go_recover` on this thread.
#[must_use]
pub fn in_go_recover() -> bool {
    GO_RECOVER_DEPTH.with(|depth| depth.get() > 0)
}



/// `std::thread::Builder` for a thread that runs work Go runs on
/// goroutines. A start that the OS refuses goes as a Go runtime thread
/// start goes (`newosproc`): it tries again while the error is EAGAIN, then
/// ends the process with Go's text (`go_fatal_newosproc`). Do not start one
/// while this thread holds a lock that one of its thread-local destructors
/// takes: the exit runs those destructors (glibc `exit`), and the process
/// hangs.
#[derive(Default)]
pub struct GoThread {
    name: Option<String>,
    stack_size: Option<usize>,
}

impl GoThread {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// `std::thread::Builder::name`.
    #[must_use]
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// `std::thread::Builder::stack_size`.
    #[must_use]
    pub fn stack_size(mut self, size: usize) -> Self {
        self.stack_size = Some(size);
        self
    }




    /// wasm32-wasip1 has no threads: `Builder::spawn` always fails there
    /// with an `Unsupported` error, which is not EAGAIN, so the start ends
    /// the process at once with the same text. This leaves std's thread
    /// start and the thread functions out of the wasm module.
    // PORT: not in Go.
    #[cfg(target_family = "wasm")]
    #[track_caller]
    fn start<'a, F, T, H>(
        &self,
        _f: F,
        _clone: impl FnMut(std::thread::Builder, ThreadMain<'a, T>) -> std::io::Result<H>,
        _sleep: impl FnMut(std::time::Duration),
    ) -> H
    where
        F: FnOnce() -> T + Send + 'a,
        T: 'a,
    {
        go_fatal_newosproc(&std::io::ErrorKind::Unsupported.into())
    }

}

/// The function of one try to start a thread (`GoThread::start`).
type ThreadMain<'a, T> = Box<dyn FnOnce() -> T + Send + 'a>;


// Go: runtime/retry.go:14 retryOnEAGAIN (go1.27.1)
// retryOnEAGAIN retries a function until it does not return EAGAIN.
// It will use an increasing delay between calls, and retry up to 20 times.
// The function argument is expected to return an errno value,
// and retryOnEAGAIN will return any errno value other than EAGAIN.
// If all retries return EAGAIN, then retryOnEAGAIN will return EAGAIN.
// PORT: Go `newosproc` (runtime/os_linux.go:170) calls it with `clone`, and
// throws on the error it returns (`GoThread::start`). `sleep` is Go
// `usleep_no_g`. Go on Windows tries again on ERROR_ACCESS_DENIED instead
// (runtime/os_windows.go:794 createThread); the port does not.
fn retry_on_eagain<H>(
    mut f: impl FnMut() -> std::io::Result<H>,
    mut sleep: impl FnMut(std::time::Duration),
) -> std::io::Result<H> {
    let mut tries = 0;
    loop {
        match f() {
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                tries += 1;
                sleep(std::time::Duration::from_millis(tries)); // milliseconds
                if tries == 20 {
                    return Err(err);
                }
            }
            result => return result,
        }
    }
}

/// `go_panic` with the Go runtime text for a nil pointer dereference, at a
/// site where the pinned Go dereferences nil on the same input. It is cold
/// and out of line, so the nil check at a hot site is one compare.
#[cold]
#[inline(never)]
#[track_caller]
pub fn go_nil_dereference() -> ! {
    go_panic("runtime error: invalid memory address or nil pointer dereference".to_string())
}

/// The Go runtime exit code after a panic that nothing recovers.
pub const EXIT_GO_PANIC: i32 = 2;



#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};
    use std::time::Duration;

    fn eagain() -> Error {
        Error::from(ErrorKind::WouldBlock)
    }

    fn ms(range: std::ops::RangeInclusive<u64>) -> Vec<Duration> {
        range.map(Duration::from_millis).collect()
    }

    // Go: runtime/retry.go:14 retryOnEAGAIN: 20 calls while the error is
    // EAGAIN, with a sleep of 1, 2, ... 20 ms after each, then EAGAIN.
    #[test]
    fn retry_on_eagain_tries_20_times_with_growing_sleeps() {
        let mut calls = 0;
        let mut sleeps = Vec::new();
        let result: std::io::Result<()> = retry_on_eagain(
            || {
                calls += 1;
                Err(eagain())
            },
            |d| sleeps.push(d),
        );
        assert_eq!(result.unwrap_err().kind(), ErrorKind::WouldBlock);
        assert_eq!(calls, 20);
        assert_eq!(sleeps, ms(1..=20));
    }

    #[test]
    fn retry_on_eagain_returns_another_error_or_a_start_at_once() {
        let mut sleeps = Vec::new();
        let result: std::io::Result<()> = retry_on_eagain(
            || Err(Error::from(ErrorKind::OutOfMemory)),
            |d| sleeps.push(d),
        );
        assert_eq!(result.unwrap_err().kind(), ErrorKind::OutOfMemory);
        assert!(sleeps.is_empty());

        let mut calls = 0;
        let result = retry_on_eagain(
            || {
                calls += 1;
                if calls < 20 { Err(eagain()) } else { Ok(calls) }
            },
            |d| sleeps.push(d),
        );
        assert_eq!(result.unwrap(), 20);
        assert_eq!(sleeps, ms(1..=19));
    }

    // A failed `Builder::spawn` drops the function it was given. The
    // function of the thread stays in the slot across the failed starts,
    // and only the started thread runs it, once.
    #[test]
    fn go_thread_keeps_the_function_across_failed_starts() {
        let runs = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let counted = runs.clone();
        let mut tries = 0;
        let mut sleeps = Vec::new();
        let handle = GoThread::new().name("retry-test".to_string()).start(
            move || {
                counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                std::thread::current().name().map(str::to_string)
            },
            |builder, run| {
                tries += 1;
                if tries <= 5 {
                    drop(run);
                    return Err(eagain());
                }
                builder.spawn(run)
            },
            |d| sleeps.push(d),
        );
        assert_eq!(handle.join().unwrap().as_deref(), Some("retry-test"));
        assert_eq!(runs.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(tries, 6);
        assert_eq!(sleeps, ms(1..=5));
    }

    #[test]
    fn go_thread_scoped_keeps_the_function_across_failed_starts() {
        let mut value = 0;
        let mut tries = 0;
        let mut sleeps = Vec::new();
        std::thread::scope(|scope| {
            let handle = GoThread::new().start(
                || {
                    value += 1;
                    value
                },
                |builder, run| {
                    tries += 1;
                    if tries <= 19 {
                        drop(run);
                        return Err(eagain());
                    }
                    builder.spawn_scoped(scope, run)
                },
                |d| sleeps.push(d),
            );
            assert_eq!(handle.join().unwrap(), 1);
        });
        assert_eq!(value, 1);
        assert_eq!(tries, 20);
        assert_eq!(sleeps, ms(1..=19));
    }
}
