// expect: L0001
// rustc E0658: a cfg_attr whose condition is off still counts as an
// attribute on the assignment expression.
fn main() {
    let mut k = 0;
    #[cfg_attr(any(), allow(unused))]
    k += 1;
    println!("{k}");
}
