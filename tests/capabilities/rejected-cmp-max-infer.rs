// capability: rejected: `cmp::max`/`cmp::min` on this type (one that a later use fixes)
// expect: L9001

fn main() {
    let a = Default::default();
    let m = std::cmp::max(a, a);
    let n: u64 = m;
    println!("{n}");
}
