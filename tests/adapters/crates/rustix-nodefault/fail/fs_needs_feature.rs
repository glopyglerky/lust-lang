// expect: L0102
// rustc E0433: `fs` is behind upstream's `fs` feature.
fn main() {
    println!("{}", rustix::fs::Mode::RUSR.bits());
}
