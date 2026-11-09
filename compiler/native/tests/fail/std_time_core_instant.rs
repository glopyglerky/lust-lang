// expect: L0505
// rustc E0433: `core::time` has no `Instant`; only `std::time` does.
fn main() {
    let t = core::time::Instant::now();
    println!("{:?}", t.elapsed());
}
