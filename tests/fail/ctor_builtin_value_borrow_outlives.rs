// expect: L0311
// rustc E0505: `Some` at a generic Fn bound returns what it takes, so
// `o` borrows `data` and the move ends that borrow too early.
fn keep<T, R, F: Fn(T) -> R>(f: F, x: T) -> R {
    f(x)
}
fn main() {
    let data = vec![1u8, 2];
    let o = keep(Some, &data[0]);
    drop(data);
    println!("{:?}", o);
}
