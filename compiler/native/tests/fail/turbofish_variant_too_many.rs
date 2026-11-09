// expect: L0105
// rustc E0107: `E` takes one argument.
enum E<T> {
    A(T),
    B,
}

fn main() {
    let e = E::<u8, u8>::B;
    println!("{}", matches!(e, E::B));
}
