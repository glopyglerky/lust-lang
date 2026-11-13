// expect: L0306
// rustc E0597: the lossy Cow borrows the byte buffer, which dies first.
fn main() {
    let c;
    {
        let v = vec![104u8, 105];
        c = String::from_utf8_lossy(&v);
    }
    println!("{}", c);
}
