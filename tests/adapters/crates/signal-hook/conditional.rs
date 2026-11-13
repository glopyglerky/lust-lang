// signal-hook 0.4.4: conditional default with the condition off
// and on an ignored signal, plus emulate_default_handler errors.
use signal_hook::consts::{SIGURG, SIGUSR1};
use signal_hook::flag;
use signal_hook::low_level::{emulate_default_handler, raise};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

fn main() {
    flag::register_conditional_default(SIGUSR1, Arc::new(AtomicBool::new(false))).unwrap();
    raise(SIGUSR1).unwrap();
    std::thread::sleep(Duration::from_millis(20));
    println!("alive1");
    flag::register_conditional_default(SIGURG, Arc::new(AtomicBool::new(true))).unwrap();
    raise(SIGURG).unwrap();
    std::thread::sleep(Duration::from_millis(20));
    println!("alive2");
    println!("{:?}", emulate_default_handler(SIGURG).map_err(|e| e.kind()));
    println!("{}", emulate_default_handler(64).unwrap_err());
    println!("{}", flag::register_conditional_default(64, Arc::new(AtomicBool::new(true))).unwrap_err());
}
