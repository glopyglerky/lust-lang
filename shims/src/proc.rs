//! Processes, limits, prctl, signal masks, and signal-hook state.
//!
//! Signal handlers in this shim never run caller code. flag register
//! flips a shim AtomicBool. A Signals iterator is signal-hook's
//! self-pipe queue behind a handle; the caller drains it. Blocking
//! drains run outside the handle-table lock so a close from another
//! thread can join and end them.

use std::ffi::c_int;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::{Handles, OK, ShimErr, ShimResult, guard, put, status};

const EINVAL_RAW: i32 = 22;

/// A rustix Signal for a raw host signal number. ts-rust kills and
/// forwards only named signals, so an unnamed number is EINVAL here.
fn signal_of(raw: i32) -> ShimResult<rustix::process::Signal> {
    rustix::process::Signal::from_named_raw(raw).ok_or_else(|| ShimErr::errno(EINVAL_RAW, "unnamed signal number"))
}

/// A rustix Pid for a raw pid. Zero and negative values are invalid:
/// every ts-rust call passes a real process id.
fn pid_of(raw: i32) -> ShimResult<rustix::process::Pid> {
    rustix::process::Pid::from_raw(raw).ok_or_else(|| ShimErr::invalid("pid"))
}

// ---------------------------------------------------------------------
// Processes and limits (rustix process)
// ---------------------------------------------------------------------

/// getpid of this process.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_getpid(out_pid: *mut i32) -> c_int {
    status(|| unsafe { put(out_pid, rustix::process::getpid().as_raw_pid(), "out_pid") })
}

/// getppid of this process. 0 means no parent (rustix gives None).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_getppid(out_pid: *mut i32) -> c_int {
    status(|| {
        let raw = rustix::process::getppid().map_or(0, |p| p.as_raw_pid());
        unsafe { put(out_pid, raw, "out_pid") }
    })
}

/// kill_process for one pid and one named signal.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_kill(pid: i32, signal: i32) -> c_int {
    status(|| {
        rustix::process::kill_process(pid_of(pid)?, signal_of(signal)?)?;
        Ok(())
    })
}

/// test_kill_process: alive gives OK, a gone process gives ESRCH.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_test_kill(pid: i32) -> c_int {
    status(|| {
        rustix::process::test_kill_process(pid_of(pid)?)?;
        Ok(())
    })
}

/// waitid(P_PID, WEXITED | WNOWAIT), the one wait form ts-rust uses.
/// The child stays unreaped; the caller still waits for its status.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_waitid_exited_nowait(pid: i32) -> c_int {
    status(|| {
        use rustix::process::{WaitId, WaitIdOptions, waitid};
        let options = WaitIdOptions::EXITED | WaitIdOptions::NOWAIT;
        waitid(WaitId::Pid(pid_of(pid)?), options)?;
        Ok(())
    })
}

/// prctl PR_SET_PDEATHSIG through rustix. Signal 0 clears it.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_set_parent_death_signal(signal: i32) -> c_int {
    status(|| {
        let sig = if signal == 0 { None } else { Some(signal_of(signal)?) };
        rustix::process::set_parent_process_death_signal(sig)?;
        Ok(())
    })
}

/// The C shape of an rlimit pair. Must match LustShimRlimit.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CRlimit {
    pub current: u64,
    pub maximum: u64,
    pub current_is_none: i32,
    pub maximum_is_none: i32,
}

/// The only resources ts-rust touches: NOFILE, AS, DATA.
fn resource_of(raw: u32) -> ShimResult<rustix::process::Resource> {
    use rustix::process::Resource;
    match raw {
        0 => Ok(Resource::Nofile),
        1 => Ok(Resource::As),
        2 => Ok(Resource::Data),
        _ => Err(ShimErr::invalid("resource")),
    }
}

/// getrlimit. rustix models an unlimited value as None.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_getrlimit(resource: u32, out: *mut CRlimit) -> c_int {
    status(|| {
        let l = rustix::process::getrlimit(resource_of(resource)?);
        let v = CRlimit {
            current: l.current.unwrap_or(0),
            maximum: l.maximum.unwrap_or(0),
            current_is_none: i32::from(l.current.is_none()),
            maximum_is_none: i32::from(l.maximum.is_none()),
        };
        unsafe { put(out, v, "out") }
    })
}

/// setrlimit.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_setrlimit(resource: u32, limit: *const CRlimit) -> c_int {
    status(|| {
        if limit.is_null() {
            return Err(ShimErr::invalid("limit"));
        }
        let l = unsafe { *limit };
        let v = rustix::process::Rlimit {
            current: (l.current_is_none == 0).then_some(l.current),
            maximum: (l.maximum_is_none == 0).then_some(l.maximum),
        };
        rustix::process::setrlimit(resource_of(resource)?, v)?;
        Ok(())
    })
}

// ---------------------------------------------------------------------
// THP prctl and the thread signal mask (nix)
// ---------------------------------------------------------------------

/// prctl PR_GET_THP_DISABLE (thp_guard).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_prctl_get_thp_disable(out_disabled: *mut i32) -> c_int {
    status(|| {
        let v = nix::sys::prctl::get_thp_disable()?;
        unsafe { put(out_disabled, i32::from(v), "out_disabled") }
    })
}

/// prctl PR_SET_THP_DISABLE (thp_guard).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_prctl_set_thp_disable(disabled: i32) -> c_int {
    status(|| {
        nix::sys::prctl::set_thp_disable(disabled != 0)?;
        Ok(())
    })
}

/// A nix Signal for a raw host number, or EINVAL. Nix has no real-time
/// signals, as ts-rust notes for its unblock list.
fn nix_signal_of(raw: i32) -> ShimResult<nix::sys::signal::Signal> {
    nix::sys::signal::Signal::try_from(raw).map_err(|_| ShimErr::errno(EINVAL_RAW, "unknown signal number for nix"))
}

/// Unblocks a set of signals on this thread (unblock_go_signals).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_thread_unblock_signals(signals: *const i32, count: usize) -> c_int {
    status(|| {
        if signals.is_null() && count > 0 {
            return Err(ShimErr::invalid("signals"));
        }
        let raw: &[i32] = if count == 0 { &[] } else { unsafe { std::slice::from_raw_parts(signals, count) } };
        let mut set = nix::sys::signal::SigSet::empty();
        for &s in raw {
            set.add(nix_signal_of(s)?);
        }
        set.thread_unblock()?;
        Ok(())
    })
}

// ---------------------------------------------------------------------
// Signal flags (signal-hook flag)
// ---------------------------------------------------------------------

static FLAGS: Handles<Arc<AtomicBool>> = Handles::new();

/// Clones the flag Arc for one handle.
fn flag_of(handle: u64) -> ShimResult<Arc<AtomicBool>> {
    FLAGS.with(handle, "signal flag handle", |f| Ok(f.clone()))
}

/// A new flag with an initial value.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_signal_flag_new(initial: i32, out_handle: *mut u64) -> c_int {
    status(|| {
        let id = FLAGS.insert(Arc::new(AtomicBool::new(initial != 0)));
        unsafe { put(out_handle, id, "out_handle") }
    })
}

/// The current flag value.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_signal_flag_get(handle: u64, out_value: *mut i32) -> c_int {
    status(|| {
        let v = flag_of(handle)?.load(Ordering::SeqCst);
        unsafe { put(out_value, i32::from(v), "out_value") }
    })
}

/// Sets the flag value (end_default_actions clears one this way).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_signal_flag_set(handle: u64, value: i32) -> c_int {
    status(|| {
        flag_of(handle)?.store(value != 0, Ordering::SeqCst);
        Ok(())
    })
}

/// Drops the shim reference. A registered handler keeps its own Arc,
/// as signal-hook does, so the flag lives while the handler does.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_signal_flag_free(handle: u64) -> c_int {
    status(|| {
        FLAGS.remove(handle, "signal flag handle")?;
        Ok(())
    })
}

/// flag::register against a flag handle. The returned SigId is not
/// exposed: ts-rust drops it at every call site (handlers stay for the
/// process life, as there).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_signal_register_flag(signal: i32, flag: u64) -> c_int {
    status(|| {
        signal_hook::flag::register(signal, flag_of(flag)?).map_err(|e| ShimErr::msg(format!("signal register: {e}")))?;
        Ok(())
    })
}

/// flag::register_conditional_default against a flag handle.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_signal_register_conditional_default(signal: i32, flag: u64) -> c_int {
    status(|| {
        signal_hook::flag::register_conditional_default(signal, flag_of(flag)?)
            .map_err(|e| ShimErr::msg(format!("signal register conditional default: {e}")))?;
        Ok(())
    })
}

// ---------------------------------------------------------------------
// Signal iterators (signal-hook iterator, a self-pipe queue)
// ---------------------------------------------------------------------

/// One Signals queue. The close Handle sits outside the Mutex so any
/// thread can end a blocked drain without the drain lock. The stash
/// keeps batch signals past a drain cap: a signal-hook Pending batch
/// is a snapshot, so dropping it early would lose the rest.
struct SignalQueue {
    inner: Mutex<(signal_hook::iterator::Signals, std::collections::VecDeque<i32>)>,
    close: signal_hook::iterator::Handle,
}

static QUEUES: Handles<Arc<SignalQueue>> = Handles::new();

/// Clones the queue Arc for one handle.
fn queue_of(handle: u64) -> ShimResult<Arc<SignalQueue>> {
    QUEUES.with(handle, "signals handle", |q| Ok(q.clone()))
}

/// Signals::new over the given host signal numbers.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_signals_new(signals: *const i32, count: usize, out_handle: *mut u64) -> c_int {
    status(|| {
        if signals.is_null() && count > 0 {
            return Err(ShimErr::invalid("signals"));
        }
        let raw: &[i32] = if count == 0 { &[] } else { unsafe { std::slice::from_raw_parts(signals, count) } };
        let signals = signal_hook::iterator::Signals::new(raw.iter().copied()).map_err(|e| ShimErr::msg(format!("signals new: {e}")))?;
        let close = signals.handle();
        let inner = Mutex::new((signals, std::collections::VecDeque::new()));
        let id = QUEUES.insert(Arc::new(SignalQueue { inner, close }));
        unsafe { put(out_handle, id, "out_handle") }
    })
}

/// Signals::add_signal after creation (forward_signals adds SIGHUP).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_signals_add(handle: u64, signal: i32) -> c_int {
    status(|| {
        let q = queue_of(handle)?;
        let inner = q.inner.lock().unwrap_or_else(|p| p.into_inner());
        inner.0.add_signal(signal).map_err(|e| ShimErr::msg(format!("signals add: {e}")))?;
        Ok(())
    })
}

/// Shared batch drain: writes up to cap signal numbers. Signals past
/// cap stay in the queue for the next drain, because take stops early.
fn drain_batch(handle: u64, out: *mut i32, cap: usize, out_n: *mut usize, wait: bool) -> ShimResult<()> {
    if cap == 0 || out.is_null() {
        return Err(ShimErr::invalid("out"));
    }
    let q = queue_of(handle)?;
    let mut found: Vec<i32> = Vec::new();
    {
        let mut inner = q.inner.lock().unwrap_or_else(|p| p.into_inner());
        let (signals, stash) = &mut *inner;
        if wait && stash.is_empty() {
            stash.extend(signals.wait());
        } else {
            stash.extend(signals.pending());
        }
        while found.len() < cap {
            let Some(s) = stash.pop_front() else { break };
            found.push(s);
        }
    }
    for (i, s) in found.iter().enumerate() {
        unsafe { *out.add(i) = *s };
    }
    unsafe { put(out_n, found.len(), "out_n") }
}

/// Blocking batch drain (signal-hook wait).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_signals_wait_batch(handle: u64, out: *mut i32, cap: usize, out_n: *mut usize) -> c_int {
    status(|| drain_batch(handle, out, cap, out_n, true))
}

/// Non-blocking batch drain (signal-hook pending).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_signals_pending_batch(handle: u64, out: *mut i32, cap: usize, out_n: *mut usize) -> c_int {
    status(|| drain_batch(handle, out, cap, out_n, false))
}

/// One blocking signal (signal-hook forever next), or none at close.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_signals_next(handle: u64, out_signal: *mut i32, out_has: *mut i32) -> c_int {
    match guard(|| {
        let q = queue_of(handle)?;
        let got = {
            let mut inner = q.inner.lock().unwrap_or_else(|p| p.into_inner());
            let (signals, stash) = &mut *inner;
            match stash.pop_front() {
                Some(s) => Some(s),
                None => signals.forever().next(),
            }
        };
        match got {
            Some(s) => {
                unsafe { put(out_signal, s, "out_signal") }?;
                unsafe { put(out_has, 1, "out_has") }
            }
            None => unsafe { put(out_has, 0, "out_has") },
        }
    }) {
        Ok(()) => OK,
        Err(s) => s,
    }
}

/// Handle::close: ends blocked drains on other threads.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_signals_close(handle: u64) -> c_int {
    status(|| {
        queue_of(handle)?.close.close();
        Ok(())
    })
}

/// Removes the queue. A drain still blocked on another thread keeps
/// its Arc alive and ends at close or at the next signal. Call
/// signals_close first, as ts-rust stop functions do.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_signals_free(handle: u64) -> c_int {
    status(|| {
        QUEUES.remove(handle, "signals handle")?;
        Ok(())
    })
}

/// low_level emulate_default_handler. Can end the process.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_signal_emulate_default(signal: i32) -> c_int {
    status(|| {
        signal_hook::low_level::emulate_default_handler(signal).map_err(|e| ShimErr::msg(format!("emulate default handler: {e}")))?;
        Ok(())
    })
}
