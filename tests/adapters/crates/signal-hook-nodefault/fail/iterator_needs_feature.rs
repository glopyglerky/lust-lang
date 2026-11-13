// expect: L0102
// rustc E0433: `iterator` is behind upstream's `iterator` feature.
fn main() {
    let s = signal_hook::iterator::Signals::new([signal_hook::consts::SIGUSR1]);
    println!("{}", s.is_ok());
}
