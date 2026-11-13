// A global where clause never guides inference (rustc): the literal
// falls back to i32, so the `Tr<i32>` impl runs, though the where
// clause names `Tr<u8>`. Lust once inferred u8 from the clause.
trait Tr<T> {
    fn f(&self, t: T) -> i32;
    fn dflt(&self) -> i32 {
        100
    }
}

struct A;
impl Tr<i32> for A {
    fn f(&self, _t: i32) -> i32 {
        1
    }
}
impl Tr<u8> for A {
    fn f(&self, _t: u8) -> i32 {
        2
    }
}

fn g() -> i32
where
    A: Tr<u8>,
{
    A.f(5)
}

fn main() {
    println!("{}", g());
}
