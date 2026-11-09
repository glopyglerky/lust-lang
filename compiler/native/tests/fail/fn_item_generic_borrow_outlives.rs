// expect: L0311
// rustc E0505: a generic fn item `id<T>(x: T) -> T` at a generic Fn
// bound carries its input's borrow to its output, as a closure does.
fn id<T>(x: T) -> T {
    x
}
fn keep<T, R, F: Fn(T) -> R>(f: F, x: T) -> R {
    f(x)
}
fn main() {
    let data = vec![1u8, 2];
    let o = keep(id, &data[0]);
    drop(data);
    println!("{:?}", o);
}
