// expect: L0306
// rustc E0597: the bytes view borrows the String behind the str.
fn main() {
    let r: &[u8];
    {
        let k = String::from("q");
        r = k.as_str().as_bytes();
    }
    println!("{r:?}");
}
