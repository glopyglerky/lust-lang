// expect: L0110
// rustc E0308: `modified` gives a SystemTime, not a number of seconds.
fn main() {
    let md = std::fs::metadata("/").unwrap();
    let t: u64 = md.modified().unwrap();
    println!("{}", t);
}
