// expect: L0110
// rustc E0508: the where clause makes `[T]` the by-value Self of `into`,
// so the call moves out of the slice.
fn f<T, U>(v: &Vec<T>) -> U
where
    [T]: Into<U>,
{
    v[1..].into()
}
fn main() {
    println!("{}", 1);
}
