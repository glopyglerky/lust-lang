// expect: L0615
// rustc E0119: a body impl conflicts with a module impl (coherence is global).
struct S;
trait T { fn t(&self) {} }
impl T for S {}
fn main() {
    impl T for S {}
    S.t();
}
