mod m {
    pub fn get(x: u32) -> u32 { x + 1 }
}
#[derive(Clone, Copy)]
struct A(u32);
struct B<T>(T);
trait G { fn get(&self) -> u32; }
struct C;
impl A {
    fn get(&self) -> u32 where u32: Copy { self.0 }
}
impl<T: Copy + Into<u32>> B<T> {
    fn get(&self) -> u32 { self.0.into() }
}
impl G for C {
    fn get(&self) -> u32 { 7 }
}
fn use_a(a: &A) -> u32 { a.get() + A::get(a) }
fn use_box(a: Box<A>) -> u32 { a.get() }
fn use_b(b: &B<u8>) -> u32 { b.get() + B::<u8>::get(b) }
fn use_c(c: &C) -> u32 { c.get() + G::get(c) }
fn use_m() -> u32 { m::get(1) }
fn run<T: Clone>(t: T) -> T { t.clone() }
fn main() {
    let a = A(2);
    println!("{} {} {} {} {} {}", use_a(&a), use_box(Box::new(a)), use_b(&B(3u8)), use_c(&C), use_m(), run(5u8));
}
