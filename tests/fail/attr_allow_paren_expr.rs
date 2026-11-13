// expect: L0001
// rustc E0658: a parenthesized expression takes no outer attribute.
fn main() {
    let v = (#[allow(unused)] 1);
    println!("{v}");
}
