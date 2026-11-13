// ARCH 2.5: a projection blocked on an inference variable at a call
// (`<?A as Array>::Item`) waits as an associated equality until the
// other argument names `A`, then normalizes (rustc's normalization
// to an inference variable).
trait Array {
    type Item;
}
impl<T> Array for [T; 4] {
    type Item = T;
}
fn put<A: Array>(_a: A, x: A::Item) -> A::Item {
    x
}
fn main() {
    let r = put([1u8; 4], 250);
    let s = put([0i16; 4], -3);
    println!("{} {}", r.wrapping_add(10), s.abs());
}
