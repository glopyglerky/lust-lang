// `nix::sys`: only the fanotify, prctl, signal, time and event
// modules are here. The fanotify and prctl forms are Linux only
// upstream too; off Linux the modules are absent, as upstream's
// are there. The signal set is Linux's (its Signal numbers are
// Linux values), so it too is absent off Linux rather than wrong
// there. The event module (kqueue) is Darwin's, as upstream's is
// BSD-only.

#[cfg(all(feature = "fanotify", target_os = "linux"))]
pub mod fanotify;
#[cfg(all(feature = "process", target_os = "linux"))]
pub mod prctl;
#[cfg(target_os = "linux")]
pub mod signal;
pub mod time;
#[cfg(all(feature = "event", target_os = "macos"))]
pub mod event;
