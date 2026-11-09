// expect: L0306
// rustc E0597: bytes borrows the String.
fn main() {
    let b;
    {
        let s = String::from("ab");
        b = s.bytes();
    }
    println!("{}", b.count());
}
