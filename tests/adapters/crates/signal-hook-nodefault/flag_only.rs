// The flag and consts forms need no feature.
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() {
    let f = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGUSR1, Arc::clone(&f)).unwrap();
    signal_hook::low_level::raise(signal_hook::consts::SIGUSR1).unwrap();
    println!("{}", f.load(Ordering::SeqCst));
}
