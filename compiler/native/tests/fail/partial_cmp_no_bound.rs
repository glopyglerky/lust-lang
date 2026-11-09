// expect: L0104
// rustc E0599: no `partial_cmp` without a `PartialOrd` bound.
fn f<T>(a: &T, b: &T) -> bool {
    a.partial_cmp(b).is_some()
}

fn main() {
    println!("{}", f(&1u8, &2u8));
}
