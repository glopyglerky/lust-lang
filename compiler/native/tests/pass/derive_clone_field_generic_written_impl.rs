// A field type whose Clone impl is generic (`G<T>`): the derived
// clone glue calls that impl's instance at the concrete type.
struct G<T>(T);
impl<T: Clone> Clone for G<T> {
    fn clone(&self) -> G<T> {
        G(self.0.clone())
    }
}
#[derive(Clone)]
struct S {
    g: G<String>,
}
fn main() {
    let s = S { g: G("ok".to_string()) };
    let t = s.clone();
    println!("{}", t.g.0);
}
