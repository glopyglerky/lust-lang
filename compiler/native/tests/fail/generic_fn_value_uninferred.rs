// expect: L0610
// rustc E0282: the value `id` leaves `T` unknown.
fn id<T>(x: T) -> T {
    x
}
fn main() {
    let f = id;
    let _ = f;
}
