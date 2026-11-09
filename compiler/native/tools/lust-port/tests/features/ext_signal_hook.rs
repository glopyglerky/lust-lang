// feature: ext.signal_hook
// form: flag registration with a self-raised signal
// cargo-dep: signal-hook = "=0.4.4"
use signal_hook::consts::SIGUSR1;
use signal_hook::low_level::raise;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
fn main() {
    let got = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(SIGUSR1, Arc::clone(&got)).unwrap();
    raise(SIGUSR1).unwrap();
    for _ in 0..2000 {
        if got.load(Ordering::Relaxed) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    println!("{}", got.load(Ordering::Relaxed));
}
