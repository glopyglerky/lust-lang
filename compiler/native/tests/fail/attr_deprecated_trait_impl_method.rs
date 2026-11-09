// expect: L0524
// rustc: the `deprecated` attribute cannot be used on trait methods in impl blocks.
struct S;
impl Clone for S {
    #[deprecated]
    fn clone(&self) -> S { S }
}
fn main() { println!("ok"); }
