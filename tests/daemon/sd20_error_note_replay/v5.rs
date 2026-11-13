// sd20
mod m { pub const R: u32 = 1; }
struct W<T>(T);
static S: fn(u32) -> W<u16> = f;
fn f(x: u32) -> W<u16> { W(x as u16) }
fn f2(x: u32) -> u32 { x }
fn g() -> usize {
    let w = f(1);
    w.0 as usize + f2(2) as usize + m::R as usize
}
fn h<T: Copy>(t: T) -> T { t }
fn main() { let p = S; println!("{} {}", g(), h(3) + p(1).0); }
