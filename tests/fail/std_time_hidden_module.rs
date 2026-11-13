// expect: L0505
// rustc E0433: lust's hidden std module name is not a path users can write.
fn main() {
    let t = __lust_std_time::Instant::now();
    println!("{:?}", t.elapsed());
}
