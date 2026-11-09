// `signal_hook::iterator`: the Signals iterator over a
// self-pipe registry. Pending batches come out sorted by
// signal number with duplicates removed, as upstream's
// slot exfiltrator yields them.

use crate::{check_forbidden, reg_status};
use std::borrow::Borrow;
use std::io;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub mod exfiltrator;

use exfiltrator::{Exfiltrator, SignalOnly};

/// The shared pipe pair of one Signals instance. The marker gives
/// `Signals`, `Handle`, and `Forever` upstream's auto traits (Send and
/// Sync, but not UnwindSafe or RefUnwindSafe: upstream holds `dyn`
/// trait objects there).
struct Inner {
    read_fd: i32,
    write_fd: i32,
    _unwind: PhantomData<Box<dyn Fn() + Send + Sync>>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        core::intrinsics::sys_sig_unregister_pipe(self.write_fd);
        let _ = core::intrinsics::sys_fd_close(self.read_fd);
        let _ = core::intrinsics::sys_fd_close(self.write_fd);
    }
}

/// A handle to a Signals instance, to close it from another
/// thread.
#[derive(Clone)]
pub struct Handle {
    inner: Arc<Inner>,
    closed: Arc<AtomicBool>,
}

impl Handle {
    /// Closes the instance: a waiting iterator ends.
    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
        // A zero byte wakes the poll; it is the close sentinel.
        let _ = core::intrinsics::sys_fd_write(self.inner.write_fd, &[0u8]);
    }

    /// True after close.
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }
}

/// An iterator of signals received since the previous call.
pub struct Signals {
    inner: Arc<Inner>,
    closed: Arc<AtomicBool>,
    queue: Vec<i32>,
    head: usize,
}

impl Signals {
    /// Registers handlers for the given signals and starts the
    /// collection. Upstream's generics: any `Borrow<c_int>` items.
    pub fn new<I, S>(signals: I) -> io::Result<Signals>
    where
        I: IntoIterator<Item = S>,
        S: Borrow<i32>,
    {
        let mut raw: Vec<u8> = Vec::new();
        let read_fd = core::intrinsics::sys_sig_pipe(&mut raw);
        if read_fd < 0 {
            return Err(io::Error::from_raw_os_error(-read_fd));
        }
        let write_fd = {
            let mut v = 0u64;
            for k in 0..8 {
                v |= (raw[k] as u64) << (8 * k);
            }
            v as i32
        };
        let mut this = Signals {
            inner: Arc::new(Inner { read_fd, write_fd, _unwind: PhantomData }),
            closed: Arc::new(AtomicBool::new(false)),
            queue: Vec::new(),
            head: 0,
        };
        for signal in signals {
            this.add_signal(*signal.borrow())?;
        }
        Ok(this)
    }

    /// Adds one signal to the collection.
    pub fn add_signal(&self, signal: i32) -> io::Result<()> {
        check_forbidden(signal);
        reg_status(core::intrinsics::sys_sig_register_pipe(signal, self.inner.write_fd))
    }

    /// A handle to close this instance from another thread.
    pub fn handle(&self) -> Handle {
        Handle { inner: Arc::clone(&self.inner), closed: Arc::clone(&self.closed) }
    }

    /// True after the handle was closed.
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    /// Moves freshly arrived signal bytes into the queue
    /// (sorted, deduped). With `wait`, blocks for the first.
    fn fill(&mut self, wait: bool) {
        if self.head > 0 {
            let mut rest: Vec<i32> = Vec::new();
            for i in self.head..self.queue.len() {
                rest.push(self.queue[i]);
            }
            self.queue = rest;
            self.head = 0;
        }
        let mut raw: Vec<u8> = Vec::new();
        let n = if wait {
            core::intrinsics::sys_sig_poll_read(self.inner.read_fd, &mut raw)
        } else {
            core::intrinsics::sys_sig_read_now(self.inner.read_fd, &mut raw)
        };
        if n <= 0 {
            return;
        }
        let mut got: Vec<i32> = Vec::new();
        for &b in &raw[..n as usize] {
            if b == 0 {
                self.closed.store(true, Ordering::SeqCst);
            } else if !got.contains(&(b as i32)) {
                got.push(b as i32);
            }
        }
        got.sort_unstable();
        for s in got {
            if !self.queue.contains(&s) {
                self.queue.push(s);
            }
        }
        self.queue.sort_unstable();
    }

    /// Already-received signals, without blocking.
    pub fn pending(&mut self) -> Pending<SignalOnly> {
        self.fill(false);
        let mut out: Vec<i32> = Vec::new();
        for i in self.head..self.queue.len() {
            out.push(self.queue[i]);
        }
        self.queue = Vec::new();
        self.head = 0;
        Pending { items: out, at: 0, _e: PhantomData }
    }

    /// Blocks until at least one signal arrives, then returns
    /// what is pending.
    pub fn wait(&mut self) -> Pending<SignalOnly> {
        while self.head >= self.queue.len() && !self.is_closed() {
            self.fill(true);
        }
        self.pending()
    }

    /// A blocking iterator over arriving signals.
    pub fn forever(&mut self) -> Forever<'_, SignalOnly> {
        Forever { signals: self, _e: PhantomData }
    }
}

/// The signals of one pending or wait batch. `E` is upstream's
/// exfiltrator parameter; only `SignalOnly` exists here, so the
/// Iterator impl is for it alone (generic uses reject).
pub struct Pending<E: Exfiltrator> {
    items: Vec<i32>,
    at: usize,
    _e: PhantomData<E>,
}

impl Iterator for Pending<SignalOnly> {
    type Item = i32;
    fn next(&mut self) -> Option<i32> {
        if self.at >= self.items.len() {
            return None;
        }
        let s = self.items[self.at];
        self.at += 1;
        Some(s)
    }
}

/// The blocking iterator of `forever` (see `Pending` for `E`).
pub struct Forever<'a, E: Exfiltrator> {
    signals: &'a mut Signals,
    _e: PhantomData<E>,
}

impl Iterator for Forever<'_, SignalOnly> {
    type Item = i32;
    fn next(&mut self) -> Option<i32> {
        loop {
            if self.signals.head < self.signals.queue.len() {
                let s = self.signals.queue[self.signals.head];
                self.signals.head += 1;
                return Some(s);
            }
            if self.signals.is_closed() {
                return None;
            }
            self.signals.fill(true);
            if self.signals.is_closed() && self.signals.head >= self.signals.queue.len() {
                return None;
            }
        }
    }
}
