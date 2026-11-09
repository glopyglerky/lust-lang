// An impl inside a fn body, next to a nested fn. Regression (fuzz
// realish seed 1, iters 358/477): the token scan counted the nested
// method, the Ast fn list did not, and slot_update indexed past the Ast
// list and panicked. The resolver now hoists the impl to the module
// items, so its method is in the fn list too.
struct S;
fn outer() {
fn inner() {
}
impl S {
    fn m(&self) {
    }
}
}
fn main() { outer(); }
