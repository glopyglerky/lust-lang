// expect: L0110
// rustc E0107: upstream's `Pending` takes an exfiltrator type parameter.
fn count(p: signal_hook::iterator::Pending) -> usize {
    p.count()
}

fn main() {
    let mut s = signal_hook::iterator::Signals::new([signal_hook::consts::SIGUSR1]).unwrap();
    println!("{}", count(s.pending()));
}
