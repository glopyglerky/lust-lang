// `nix::sys::prctl`: the THP disable forms over sysx prctl.

use crate::errno::Errno;
use crate::Result;

/// Sets the "THP disable" flag for the calling thread
/// (`PR_SET_THP_DISABLE`).
pub fn set_thp_disable(flag: bool) -> Result<()> {
    let e = core::intrinsics::sys_x_prctl_thp(1, if flag { 1 } else { 0 });
    if e != 0 {
        return Err(Errno::from_raw(-e));
    }
    Ok(())
}

/// Gets the "THP disable" flag (`PR_GET_THP_DISABLE`).
pub fn get_thp_disable() -> Result<bool> {
    let r = core::intrinsics::sys_x_prctl_thp(0, 0);
    if r < 0 {
        return Err(Errno::from_raw(-r));
    }
    Ok(r != 0)
}
