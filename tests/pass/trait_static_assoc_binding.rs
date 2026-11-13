// `type Req: 'static` in a `'static` trait: the impl's binding is the type
// checked, not the projection (ts-rust ls/crossproject.rs CrossProjectSearch).
trait Search: 'static {
    type Req: Clone + Send + 'static;
    fn go(r: &Self::Req) -> usize;
}
#[derive(Clone)]
struct P { v: Vec<u8> }
struct R;
impl Search for R {
    type Req = P;
    fn go(r: &Self::Req) -> usize { r.v.len() }
}
struct W<T>(T);
impl<T: Clone + Send + 'static> Search for W<T> {
    type Req = Option<T>;
    fn go(r: &Self::Req) -> usize { r.iter().count() }
}
fn run<S: Search>(r: S::Req) -> usize { S::go(&r) }
fn main() { println!("{} {}", run::<R>(P { v: vec![1, 2] }), run::<W<u8>>(Some(3))); }
