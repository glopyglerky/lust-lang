// expect: L0001
// rustc E0658: the value of an array repeat takes no outer attribute (an
// array element does).
fn main() {
    let v = [#[cfg(any())] 1 + 1; 2];
    println!("{:?}", v);
}
