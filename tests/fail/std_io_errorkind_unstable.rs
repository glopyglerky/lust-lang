// expect: L0104
// ErrorKind::Uncategorized is unstable (rustc E0658).
fn main() {
    let k = std::io::ErrorKind::Uncategorized;
    println!("{:?}", k);
}
