struct S;
trait Tr {
    fn m(&self) -> u8;
}
impl Tr for S {
    fn m(&self) -> u8 {
        1
    }
}
impl Tr for &S {
    fn m(&self) -> u8 {
        2
    }
}
fn via_bound<T: Tr>(t: &T) -> u8 {
    t.m()
}
fn concrete(r: &S) -> u8 {
    r.m()
}
fn main() {
    let s = S;
    let r = &s;
    // Generic arg: `&&S` against `&T` binds `T = &S` exactly.
    // Concrete arg: `&&S` against `&S` deref-coerces once.
    println!("{} {} {}", via_bound(&s), via_bound(&r), concrete(&r));
}
