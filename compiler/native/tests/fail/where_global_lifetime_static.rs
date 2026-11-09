// expect: L0733
// rustc E0597: only the `'static` impl proves the where clause, so the
// borrow of `a` must be 'static. Lust once proved it from the where
// clause alone (and called the trait method's stub); now the impl's
// region bound applies in the body, so lust reports it there.
trait Tr {
    fn go(&self) -> i32;
    fn dflt(&self) -> i32 {
        100
    }
}

struct A;
impl Tr for &'static A {
    fn go(&self) -> i32 {
        9
    }
}

fn f<'a>(x: &'a A) -> i32
where
    &'a A: Tr,
{
    x.go()
}

fn main() {
    let a = A;
    println!("{}", f(&a));
}
