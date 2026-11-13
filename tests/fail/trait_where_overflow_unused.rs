// expect: L0611
// rustc E0275 at the impl: proving its own where clause `G<Option<T>>: Tr`
// evaluates the impl again at a larger type, past the recursion limit.
// Nothing uses the impl.
struct G<T>(T);
trait Tr {}
impl<T> Tr for G<T> where G<Option<T>>: Tr {}
fn main() {
    println!("ok");
}
