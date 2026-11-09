// expect: L0612
// rustc E0277: the unused struct is accepted (the bound over
// two defaults is lazy), but using the defaults owes it.
trait Pair<B> {}
struct Bad<A: Pair<B> = u8, B = u16>(A, B);
fn main() {
    let _x: Option<Bad> = None;
}
