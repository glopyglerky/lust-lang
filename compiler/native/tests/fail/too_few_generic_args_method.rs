// expect: L0107
// rustc E0107: the method takes 2 generic arguments but 1 was supplied.
struct S;
impl S { fn g<A, B>(&self, a: A, _b: B) -> A { a } }
fn main() { println!("{}", S.g::<_>(1u8, 2u8)); }
