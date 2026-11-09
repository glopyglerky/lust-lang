// signal-hook 0.4.4: flag registration and self-raised signals.
use signal_hook::consts::{SIGUSR1, SIGUSR2};
use signal_hook::flag;
use signal_hook::low_level::{raise, signal_name};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

fn wait_until<F: Fn() -> bool>(f: F) -> bool {
    for _ in 0..2000 {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    false
}

fn main() {
    let got = Arc::new(AtomicBool::new(false));
    flag::register(SIGUSR1, Arc::clone(&got)).unwrap();
    raise(SIGUSR1).unwrap();
    println!("{}", wait_until(|| got.load(Ordering::Relaxed)));
    let val = Arc::new(AtomicUsize::new(0));
    flag::register_usize(SIGUSR2, Arc::clone(&val), 42).unwrap();
    raise(SIGUSR2).unwrap();
    println!("{}", wait_until(|| val.load(Ordering::Relaxed) == 42));
    println!("{:?} {:?} {}", signal_name(9), signal_name(142), signal_hook::consts::FORBIDDEN.len());
    println!("{}", signal_hook::consts::signal::SIGTERM);
}
