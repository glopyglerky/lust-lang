// expect: L0633
// A const of a type with a hand-written PartialEq is not structural-match.
struct S(u8);
impl PartialEq for S {
    fn eq(&self, o: &S) -> bool {
        self.0 == o.0
    }
}
const A: S = S(1);
fn main() {
    match S(1) {
        A => println!("a"),
        S(_) => println!("b"),
    }
}
