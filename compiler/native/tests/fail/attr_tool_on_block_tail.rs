// expect: L0001
// rustc E0658: a tool attribute on a block tail (an expression statement
// with no `;`) is not permitted; with a `;` it is.
fn main() {
    let v = {
        #[rustfmt::skip]
        1
    };
    println!("{v}");
}
