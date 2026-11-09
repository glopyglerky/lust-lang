// expect: L0820
// A block item named like a generic parameter of the enclosing fn:
// valid Rust (the item shadows the parameter) that lust rejects on
// purpose, since a mono instance would read the name as the argument
// (this printed `0` instead of `T { a: 0 }`).
fn genf<T: Default + std::fmt::Debug>(_t: T) {
    #[derive(Debug, Default)]
    struct T { a: i8 }
    println!("{:?}", T::default());
}
fn main() { genf(1u8); }
