// expect: L0110
// rustc E0109: arguments on both the enum and the variant segment.
enum E<T> {
    A(T),
    B,
}

fn main() {
    let e = E::<u8>::A::<u8>(1);
    println!("{}", matches!(e, E::A(_)));
}
