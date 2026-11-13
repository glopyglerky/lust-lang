// expect: L0301
// rustc E0382: `map` takes the Option by value.
fn main() {
    let a = Some(String::from("x"));
    let b = a.map(|s| s.len());
    println!("{:?} {:?}", a, b);
}
