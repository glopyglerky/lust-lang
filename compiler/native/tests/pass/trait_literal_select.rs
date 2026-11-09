// An integer or float literal whose type is still open selects the one
// impl that matches it; two matching impls wait for the literal fallback
// (i32, f64), as in rustc.
trait Tr { fn m(&self) -> u32; }
impl Tr for u8 { fn m(&self) -> u32 { 8 } }
trait Fl { fn t(&self) -> u32; }
impl Fl for f32 { fn t(&self) -> u32 { 32 } }
trait Both { fn b(&self) -> u32; }
impl Both for f32 { fn b(&self) -> u32 { 32 } }
impl Both for f64 { fn b(&self) -> u32 { 64 } }
trait Twice { fn twice(self) -> Self; }
impl Twice for u8 { fn twice(self) -> u8 { self.wrapping_mul(2) } }
fn g<T: Tr>(t: T) -> u32 { t.m() }
fn show<T: Fl>(x: T) -> u32 { x.t() }
fn both<T: Both>(x: T) -> u32 { x.b() }
fn tw<T: Twice>(t: T) -> T { t.twice() }
fn copy<T: Copy>(x: T) -> T { x }
fn sum<T: std::ops::Add<Output = T> + Copy>(a: T, b: T) -> T { a + b }
fn main() {
    println!("{} {} {}", 5.m(), g(5), show(2.5));
    println!("{} {}", both(1.5), both(1.5f32));
    let x = 200;
    println!("{} {}", tw(x), x);
    println!("{} {} {}", copy(0.1) + 0.2, copy(7) + 1, sum(1.5, 2.25));
}
