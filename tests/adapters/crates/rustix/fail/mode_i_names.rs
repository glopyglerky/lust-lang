// expect: L0104
// rustc E0599: upstream names the mode bits `RUSR`, not `IRUSR`.
fn main() {
    println!("{}", rustix::fs::Mode::IRUSR.bits());
}
