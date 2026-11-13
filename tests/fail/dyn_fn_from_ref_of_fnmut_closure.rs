// expect: L0612
// `&C` is Fn only when C is Fn: a closure that mutates its capture cannot be a `&dyn Fn`
// through a shared reference (rustc E0525/E0277).
fn main() {
    let mut n = 0u32;
    let mut c = |i: u32| { n += i; n };
    let d: &dyn Fn(u32) -> u32 = &c;
    println!("{}", d(1));
}
