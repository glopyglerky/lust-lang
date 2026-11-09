// `signal_hook::low_level`: raise, exit, abort, signal_name,
// and emulate_default_handler over upstream's details table.

use std::io;

/// The default-action kind of a signal (upstream's table).
#[derive(Copy, Clone, PartialEq, Eq)]
enum Kind {
    Ignore,
    Stop,
    Term,
}

/// (name, kind) of one known signal: upstream's table, by number.
fn details(signal: i32) -> Option<(&'static str, Kind)> {
    Some(match signal {
        1 => ("SIGHUP", Kind::Term),
        2 => ("SIGINT", Kind::Term),
        3 => ("SIGQUIT", Kind::Term),
        4 => ("SIGILL", Kind::Term),
        5 => ("SIGTRAP", Kind::Term),
        6 => ("SIGABRT", Kind::Term),
        7 => ("SIGBUS", Kind::Term),
        8 => ("SIGFPE", Kind::Term),
        9 => ("SIGKILL", Kind::Term),
        10 => ("SIGUSR1", Kind::Term),
        11 => ("SIGSEGV", Kind::Term),
        12 => ("SIGUSR2", Kind::Term),
        13 => ("SIGPIPE", Kind::Term),
        14 => ("SIGALRM", Kind::Term),
        15 => ("SIGTERM", Kind::Term),
        17 => ("SIGCHLD", Kind::Ignore),
        18 => ("SIGCONT", Kind::Ignore),
        19 => ("SIGSTOP", Kind::Stop),
        20 => ("SIGTSTP", Kind::Stop),
        21 => ("SIGTTIN", Kind::Stop),
        22 => ("SIGTTOU", Kind::Stop),
        23 => ("SIGURG", Kind::Ignore),
        24 => ("SIGXCPU", Kind::Term),
        25 => ("SIGXFSZ", Kind::Term),
        26 => ("SIGVTALRM", Kind::Term),
        27 => ("SIGPROF", Kind::Term),
        28 => ("SIGWINCH", Kind::Ignore),
        29 => ("SIGIO", Kind::Ignore),
        31 => ("SIGSYS", Kind::Term),
        _ => return None,
    })
}

/// A human-readable name of a signal, if known.
pub fn signal_name(signal: i32) -> Option<&'static str> {
    details(signal).map(|(name, _)| name)
}

/// The usual raise, as a safe wrapper.
pub fn raise(sig: i32) -> io::Result<()> {
    let e = core::intrinsics::sys_sig_raise(sig);
    if e == 0 { Ok(()) } else { Err(io::Error::from_raw_os_error(e)) }
}

/// A bare libc abort.
pub fn abort() -> ! {
    core::intrinsics::sys_sig_abort()
}

/// A bare libc _exit.
pub fn exit(status: i32) -> ! {
    core::intrinsics::sys_sig_exit(status)
}

/// Emulates the default action of a signal, as upstream's
/// table-driven form does.
pub fn emulate_default_handler(signal: i32) -> io::Result<()> {
    if signal == 19 || signal == 9 {
        return raise(signal);
    }
    let Some((_, kind)) = details(signal) else {
        return Err(io::Error::from_raw_os_error(22));
    };
    match kind {
        Kind::Ignore => Ok(()),
        Kind::Stop => raise(19),
        Kind::Term => {
            let e = core::intrinsics::sys_sig_restore_raise(signal);
            Err(io::Error::from_raw_os_error(e))
        }
    }
}
