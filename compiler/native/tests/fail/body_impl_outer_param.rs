// expect: L0102
// rustc E0401: a body impl may not use the enclosing fn's generic params.
fn f<T: Default>() {
    struct L;
    impl L { fn mk() -> T { T::default() } }
}
fn main() { f::<u8>(); }
