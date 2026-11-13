// A generic `impl Fn() -> T` called where it is made: the hidden
// closure's signature names the definer's `T`, which the call's args
// give (`u32` here, `String` below).
fn first<T: Copy>(v: &[T]) -> impl Fn() -> T + '_ {
    move || v[0]
}
fn owned<T: Clone>(v: Vec<T>) -> impl Fn() -> T {
    move || v[0].clone()
}
fn main() {
    let v = vec![7u32, 8];
    let f = first(&v);
    let g = owned(vec!["a".to_string()]);
    println!("{} {}", f() + 1, g().len());
}
