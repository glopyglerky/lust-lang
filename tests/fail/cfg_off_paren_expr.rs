// expect: L0001
// rustc E0658: a parenthesized expression takes no outer attribute; a tuple
// element does. lust used to drop the `1` and print `()`.
fn main() {
    let v = (#[cfg(any())] 1);
    println!("{:?}", v);
}
