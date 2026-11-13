// A derive edit must check again every template whose check reads the
// toggled trait: borrowck's Copy read in a generic fn (w-sig-front 14:41),
// and trait default bodies (templates after the AST fns) for Debug,
// Default and a header edit of a fn they call. Each reject reverts.
const ROUTE: u8 = 0;
#[derive(Clone, Copy, Debug, Default)]
struct S {
    x: u32,
}
fn h() -> u32 {
    1
}
fn dup<T>(s: S, _t: T) -> (S, S) {
    (s, s)
}
fn same<T>(s: S, _t: T) -> bool {
    let f = |a: &S| *a == s;
    f(&s)
}
struct G<T>(T);
impl<T> G<T> {
    fn twin(&self, s: &S) -> S {
        s.clone()
    }
}
trait Show {
    fn show(&self, s: S) -> String {
        format!("{:?}", s)
    }
    fn fresh(&self) -> u32 {
        let d: S = Default::default();
        d.x + h()
    }
}
impl Show for u8 {}
fn main() {
    let (a, b) = dup(S { x: 1 }, ROUTE);
    println!("{} {} {}", a.x, b.x, same(a, 'c'));
    println!("{} {} {}", G(2u8).twin(&a).x, 1u8.show(b), 1u8.fresh());
}
