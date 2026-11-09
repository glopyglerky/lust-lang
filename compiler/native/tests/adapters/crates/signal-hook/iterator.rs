// signal-hook 0.4.4: the Signals iterator (wait, pending,
// forever, handle close) with self-raised signals.
use signal_hook::consts::{SIGUSR1, SIGUSR2};
use signal_hook::iterator::Signals;
use signal_hook::low_level::raise;
use std::time::Duration;

fn main() {
    let mut signals = Signals::new([SIGUSR1, SIGUSR2]).unwrap();
    println!("{}", signals.pending().count());
    raise(SIGUSR2).unwrap();
    raise(SIGUSR1).unwrap();
    let batch: Vec<i32> = signals.wait().collect();
    println!("{batch:?}");
    let handle = signals.handle();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        raise(SIGUSR1).unwrap();
    });
    println!("{:?}", signals.forever().next());
    handle.close();
    println!("{} {:?}", handle.is_closed(), signals.forever().next());
}
