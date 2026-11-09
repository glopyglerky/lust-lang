// The keyed walk after a reject (w-sig-small round 5): v4's `+ Copy`
// refuses the header delta (hidden_diag) and fails; the put-back program
// stays (keep_err), and the read sets keep its marks, so the v5 revert
// takes the keyed walk. v2 is a body edit of the RPIT fn, v3 and v6 add
// `+ Sized`. The const item takes the resolver path.
#![allow(unused)]
const ROUTE: u8 = 0;
fn names(v: &[u32]) -> impl Iterator<Item = u32> + '_ {
    let _lust_e: u32 = 1;
    v.iter().map(|x| x + 1)
}
fn id<T: Clone>(t: T) -> T { t.clone() }
fn user1() -> u32 { names(&[1, 2]).sum() }
fn user2() -> u32 { id(3) }
fn user3() -> u32 { let c = |a: u32| a + 1; c(4) }
fn user4() -> String { format!("{}", id(5u8)) }
fn main() {
    println!("{} {} {} {}", user1(), user2(), user3(), user4());
}
