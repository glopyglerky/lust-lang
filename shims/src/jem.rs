//! jemalloc control surface (tikv-jemalloc-ctl) and the global
//! allocator choice for this cdylib.
//!
//! With `--features jemalloc`, jemalloc is the global allocator of
//! the cdylib (the `#[global_allocator]` below) and the exports read
//! and write the real `thread.arena` and `opt.narenas` controls that
//! ts-rust uses in program.rs and jemalloc_layout.rs. Without the
//! feature the cdylib uses the system allocator, `enabled` gives 0,
//! and every control gives UNSUPPORTED. A silent no-op control would
//! break the single-threaded checker arena rule in ARCH 3.7, so this
//! module never fakes one.

use std::ffi::c_int;

#[cfg(feature = "jemalloc")]
use crate::ShimResult;
use crate::{ShimErr, put, status};

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// Reads one u32 control by name. Only the two names ts-rust uses are
/// wired: thread.arena and opt.narenas (through the exports below).
#[cfg(feature = "jemalloc")]
fn ctl_read_u32(name: &'static [u8]) -> ShimResult<u32> {
    use tikv_jemalloc_ctl::{Access, AsName};
    let v: u32 = name.name().read().map_err(|e| ShimErr::msg(format!("jemalloc ctl read: {e}")))?;
    Ok(v)
}

/// Writes one u32 control by name.
#[cfg(feature = "jemalloc")]
fn ctl_write_u32(name: &'static [u8], value: u32) -> ShimResult<()> {
    use tikv_jemalloc_ctl::{Access, AsName};
    Access::<u32>::write(name.name(), value).map_err(|e| ShimErr::msg(format!("jemalloc ctl write: {e}")))
}

/// 1 when this build links jemalloc, else 0.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_jemalloc_enabled(out_enabled: *mut i32) -> c_int {
    status(|| {
        let on = i32::from(cfg!(feature = "jemalloc"));
        unsafe { put(out_enabled, on, "out_enabled") }
    })
}

/// thread.arena of this thread.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_jemalloc_thread_arena_get(out_arena: *mut u32) -> c_int {
    status(|| {
        #[cfg(feature = "jemalloc")]
        {
            let v = ctl_read_u32(b"thread.arena\0")?;
            unsafe { put(out_arena, v, "out_arena") }
        }
        #[cfg(not(feature = "jemalloc"))]
        {
            let _ = out_arena;
            Err(ShimErr::unsupported("jemalloc (build lust-shim with --features jemalloc)"))
        }
    })
}

/// Sets thread.arena of this thread. Later allocations on this thread
/// land in that arena, through the alloc bridge or any Rust code in
/// the cdylib.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_jemalloc_thread_arena_set(arena: u32) -> c_int {
    status(|| {
        #[cfg(feature = "jemalloc")]
        {
            ctl_write_u32(b"thread.arena\0", arena)
        }
        #[cfg(not(feature = "jemalloc"))]
        {
            let _ = arena;
            Err(ShimErr::unsupported("jemalloc (build lust-shim with --features jemalloc)"))
        }
    })
}

/// opt.narenas, the arena count jemalloc_layout walks.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_jemalloc_opt_narenas(out_arenas: *mut u32) -> c_int {
    status(|| {
        #[cfg(feature = "jemalloc")]
        {
            let v = ctl_read_u32(b"opt.narenas\0")?;
            unsafe { put(out_arenas, v, "out_arenas") }
        }
        #[cfg(not(feature = "jemalloc"))]
        {
            let _ = out_arenas;
            Err(ShimErr::unsupported("jemalloc (build lust-shim with --features jemalloc)"))
        }
    })
}

/// The allocation that makes an arena skip to its 8 MiB grow step.
/// 4 MiB size at 4 MiB alignment, as in ts-rust jemalloc_layout.rs.
#[cfg(feature = "jemalloc")]
#[repr(C, align(4194304))]
struct Grow([u8; 4 << 20]);

/// The jemalloc memory layout of ts-rust jemalloc_layout.rs, without
/// its caller policy: in each arena, one 4 MiB allocation at 4 MiB
/// alignment freed at once, then one parked thread named arena-hold
/// bound to the last arena. A step that fails leaves the rest as it
/// was, as there. Call at most once, at process start.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_jemalloc_layout() -> c_int {
    status(|| {
        #[cfg(feature = "jemalloc")]
        {
            use tikv_jemalloc_ctl::{Access, AsName};
            let Ok(arenas) = Access::<u32>::read(b"opt.narenas\0".name()) else {
                return Ok(());
            };
            let Ok(own) = Access::<u32>::read(b"thread.arena\0".name()) else {
                return Ok(());
            };
            for arena in 0..arenas {
                if b"thread.arena\0".name().write(arena).is_ok() {
                    let mut grow = Vec::<Grow>::new();
                    let _ = grow.try_reserve_exact(1);
                    std::hint::black_box(&mut grow);
                }
            }
            let _ = b"thread.arena\0".name().write(own);
            let Some(last) = arenas.checked_sub(1) else {
                return Ok(());
            };
            let _ = std::thread::Builder::new().name("arena-hold".to_string()).stack_size(256 << 10).spawn(move || {
                if b"thread.arena\0".name().write(last).is_ok() {
                    loop {
                        std::thread::park();
                    }
                }
            });
            Ok(())
        }
        #[cfg(not(feature = "jemalloc"))]
        {
            Err(ShimErr::unsupported("jemalloc (build lust-shim with --features jemalloc)"))
        }
    })
}
