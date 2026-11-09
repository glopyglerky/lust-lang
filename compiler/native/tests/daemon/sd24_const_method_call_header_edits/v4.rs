// Method calls `.get(..)` in const evaluation roots (values, const fn bodies, const blocks,
// array lengths, a LazyLock closure) while Ty::get's header is edited: a non-const method
// takes the header delta; one that is const before or after does not (round 5 item 2).
#![allow(dead_code, non_upper_case_globals, unused_imports)]
use std::sync::LazyLock;
struct Ty { x: u32 }
impl Ty {
    const fn get(&self, x: u32) -> u32 { self.x + x
    }
}
struct Bits(u32);
impl Bits {
    const fn get(&self, x: u32) -> u32 { self.0 + x }
}
const B1: u32 = Bits(5).get(1);
const fn cf3(b: &Bits) -> u32 { b.get(3) * 2 }
const B3: u32 = cf3(&Bits(2));
fn f6() -> u32 { const { Bits(3).get(1) } }
static S7: LazyLock<u32> = LazyLock::new(|| Ty { x: 1 }.get(4) as u32);
fn f12() -> u32 { let a = [0u8; Bits(1).get(1) as usize]; a.len() as u32 }
fn wrap<T: Copy>(t: T) -> T { t }
fn call() -> u32 { Ty { x: 1 }.get(1) as u32 }
fn main() {
    println!("{} {}", B1 + B3 + f6() + *S7 + f12(), wrap(call()));
}
