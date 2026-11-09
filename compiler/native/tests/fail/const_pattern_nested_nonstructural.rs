// expect: L0633
// Every Adt the const's value reaches must derive PartialEq.
struct M;
impl PartialEq for M {
    fn eq(&self, _: &M) -> bool {
        true
    }
}
#[derive(PartialEq)]
struct W<T>(T);
const C: W<M> = W(M);
fn main() {
    match W(M) {
        C => println!("a"),
        _ => println!("b"),
    }
}
