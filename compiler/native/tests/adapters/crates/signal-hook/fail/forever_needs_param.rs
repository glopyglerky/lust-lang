// expect: L0110
// rustc E0107: upstream's `Forever` takes a lifetime and an exfiltrator.
fn first(f: signal_hook::iterator::Forever<'_>) -> Option<i32> {
    let mut f = f;
    f.next()
}

fn main() {
    let mut s = signal_hook::iterator::Signals::new([signal_hook::consts::SIGUSR1]).unwrap();
    s.handle().close();
    println!("{:?}", first(s.forever()));
}
