// expect: L0001
// rustc E0658: `.a` continues the block into a plain expression, and the
// `+` then makes the cfg an attribute on its left operand.
struct P { a: i32 }
fn main() {
    #[cfg(all())]
    { P { a: 1 } }.a + 1;
}
