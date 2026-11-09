// The keyed const body check (w-sig-small round 6): after a header or
// derive delta only const and static initializers that read a moved key
// check again (the header delta refuses an edit of a fn a const names).
// v2 and v3 move the header of `other`, which no const reads; v4 moves
// B's trait method (E0053; AV reads the impls of Tr); v5 reverts; v6
// drops Copy from K (P rejects, E0382); v7 puts Copy back and moves the
// header of `other` again; v8 adds Debug to K (P checks again, passes).
#![allow(unused)]
#[derive(Clone, Copy, Debug)]
struct K { x: u32 }
trait Tr { const V: u32; fn m(&self, x: u32) -> u32; }
struct A;
struct B;
impl Tr for A { const V: u32 = 1; fn m(&self, x: u32) -> u32 { x } }
impl Tr for B { const V: u32 = 2; fn m(&self, x: u32) -> u32 { x + 1 } }
const fn f(x: u32) -> u32 { x + 1 }
fn h(x: u32) -> u32 { x * 2 }
fn id<T: Clone>(t: T) -> T { t.clone() }
fn other(x: u32) -> u32 { x }
const A0: u32 = f(1);
const P: (K, K) = { let k = K { x: 2 }; (k, k) };
const FP: fn(u32) -> u32 = h;
static TABLE: [fn(u32) -> u32; 2] = [h, h];
const AV: u32 = <A as Tr>::V + <B as Tr>::V;
struct W;
impl W { const D: u32 = f(2) + 1; }
fn main() {
    const N: u32 = f(3);
    let o = other(4);
    println!("{} {} {} {} {} {} {} {} {} {}", A0, P.0.x + P.1.x, FP(1), TABLE[1](2), AV, W::D, N, o, id(5u8), B.m(1));
}
