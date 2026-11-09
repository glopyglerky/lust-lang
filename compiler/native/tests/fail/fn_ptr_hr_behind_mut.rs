// expect: L0110
// rustc E0308: behind `&mut` the fn pointer types must be equal, and
// `for<'r> fn(&'r u8)` is more general than `fn(&'b u8)`.
fn k<'b>(x: &'b mut fn(&u8)) -> &'b mut fn(&'b u8) {
    x
}
fn main() {}
