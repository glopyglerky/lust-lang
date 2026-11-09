// The keyed walk after a header delta (w-sig-small round 4): only read
// sets that hold a key the delta moved compare again. f's header moves in
// v2 (a trailing comma), v3 (user1 and the closure in user2 reject,
// E0308), v4 (back to v1), and v5 (a where clause). Bodies that never
// read f keep their results. The const item takes the resolver path.
#![allow(unused)]
const ROUTE: u8 = 0;
fn f(x: u32) -> u64 { x as u64 + 1 }
fn id<T: Clone>(t: T) -> T { t.clone() }
fn user1() -> u32 { id(f(1)) }
fn user2() -> u32 { let c = |a: u32| a + f(a); c(3) }
fn other() -> u32 { let c = |a: u32| a + id(2u32); c(5) }
fn main() { let v = vec![user1(), user2(), other()]; println!("{:?}", v); }
