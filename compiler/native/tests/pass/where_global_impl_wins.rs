// A where-bound that names no generic parameter (`A: Tr` on a plain
// fn) is answered by the impl, as in rustc: the calls reach the
// impl's methods, the trait defaults, and the impl's consts.
trait Tr {
    const C: i32;
    fn go(&self) -> i32;
    fn dflt(&self) -> i32 {
        Self::C * 100
    }
}
struct A;
impl Tr for A {
    const C: i32 = 2;
    fn go(&self) -> i32 {
        1
    }
}
fn plain() -> i32
where
    A: Tr,
{
    A.go() + A.dflt() + A::C
}
fn ufcs() -> i32
where
    A: Tr,
{
    Tr::go(&A) + <A as Tr>::dflt(&A)
}
fn with_param<T: Tr>(t: T) -> i32
where
    A: Tr,
{
    t.go() + A.go() + t.dflt()
}
struct B;
impl B {
    fn m(&self) -> i32
    where
        A: Tr,
    {
        A.dflt() + A.go()
    }
}
impl A {
    fn twice(&self) -> i32
    where
        Self: Tr,
    {
        self.dflt() + Self::go(self)
    }
}
fn main() {
    println!("{} {} {} {} {}", plain(), ufcs(), with_param(A), B.m(), A.twice());
}
