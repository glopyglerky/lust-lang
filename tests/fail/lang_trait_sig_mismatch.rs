// expect: L0110
// rustc E0053: an impl method of a lang trait must match the
// trait method's types. Lang traits have no AST template; the
// check runs against the lang sig (W-STD-C phase 2).
struct W(i32);
impl Clone for W {
    fn clone(&self) -> i32 {
        self.0
    }
}
fn main() {
    let w = W(3);
    println!("{}", w.clone());
}
