// The derive delta (sigedit P3.2) over a body whose discovery check stays
// pending in `lust check` (a generic fn's opaque impl Fn* inside Option):
// collect stores no entry for it, so the delta's replay checks it again
// against the output collect kept. Hash and PartialEq go and come back,
// and a Copy drop is a reject (wrap needs T: Copy).
const _ROUTE: u8 = 0;
#[derive(Clone, Copy, Debug, PartialEq)]
struct A {
    x: u8,
}
fn wrap<T: Copy>(t: T) -> impl FnOnce() -> T {
    move || t
}
fn main() {
    let a = A { x: 5 };
    let mut o = Some(wrap(a));
    if let Some(f) = o.take() {
        println!("{:?}", f());
    }
    println!("{}", o.is_none());
}
