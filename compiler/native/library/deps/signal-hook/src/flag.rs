// `signal_hook::flag`: actions that set shared flags. The Arcs
// are leaked into the C registry (upstream keeps them in its
// registry until unregistered, which this adapter has no form
// for), so a registered flag stays live for the process.

use crate::low_level::signal_name;
use crate::{SigId, check_forbidden, next_sig_id, reg_status};
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize};

/// Registers an action to set the flag to true on the signal.
pub fn register(signal: i32, flag: Arc<AtomicBool>) -> io::Result<SigId> {
    check_forbidden(signal);
    let ptr = Arc::as_ptr(&flag) as *mut u8 as *mut ();
    let e = core::intrinsics::sys_sig_register(signal, 0, ptr, 0);
    reg_status(e)?;
    core::mem::forget(flag);
    Ok(next_sig_id())
}

/// Registers an action to set the flag to `value` on the signal.
pub fn register_usize(signal: i32, flag: Arc<AtomicUsize>, value: usize) -> io::Result<SigId> {
    check_forbidden(signal);
    let ptr = Arc::as_ptr(&flag) as *mut u64 as *mut ();
    let e = core::intrinsics::sys_sig_register(signal, 2, ptr, value as u64);
    reg_status(e)?;
    core::mem::forget(flag);
    Ok(next_sig_id())
}

/// Exits with `status` on the signal when `condition` is true
/// at delivery time.
pub fn register_conditional_shutdown(signal: i32, status: i32, condition: Arc<AtomicBool>) -> io::Result<SigId> {
    check_forbidden(signal);
    let ptr = Arc::as_ptr(&condition) as *mut u8 as *mut ();
    let e = core::intrinsics::sys_sig_register(signal, 3, ptr, status as u32 as u64);
    reg_status(e)?;
    core::mem::forget(condition);
    Ok(next_sig_id())
}

/// Runs the default action on the signal when `condition` is
/// true at delivery time. An unknown signal is EINVAL, as
/// upstream.
pub fn register_conditional_default(signal: i32, condition: Arc<AtomicBool>) -> io::Result<SigId> {
    check_forbidden(signal);
    if signal_name(signal).is_none() {
        return Err(io::Error::from_raw_os_error(22));
    }
    let ptr = Arc::as_ptr(&condition) as *mut u8 as *mut ();
    let e = core::intrinsics::sys_sig_register(signal, 1, ptr, 0);
    reg_status(e)?;
    core::mem::forget(condition);
    Ok(next_sig_id())
}
