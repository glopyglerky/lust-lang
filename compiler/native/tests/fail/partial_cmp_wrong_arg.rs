// expect: L0110
// rustc E0308: the other side of `partial_cmp` is the `Rhs` type.
fn f<T: PartialOrd>(a: &T) -> bool {
    a.partial_cmp(&1u8).is_some()
}

fn main() {
    println!("{}", f(&1u16));
}
