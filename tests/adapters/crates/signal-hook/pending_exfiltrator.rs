// The exfiltrator forms upstream has for the default `Signals`.
use signal_hook::iterator::exfiltrator::SignalOnly;
use signal_hook::iterator::{Pending, Signals};

fn count(p: Pending<SignalOnly>) -> usize {
    p.count()
}

fn main() {
    let s0: &[i32] = &[signal_hook::consts::SIGUSR1];
    let mut s = Signals::new(s0).unwrap();
    s.add_signal(signal_hook::consts::SIGUSR2).unwrap();
    println!("{}", count(s.pending()));
    println!("{:?} {}", SignalOnly, signal_hook::consts::TERM_SIGNALS.len());
    s.handle().close();
    println!("{}", s.forever().count());
}
