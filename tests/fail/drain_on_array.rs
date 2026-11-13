// expect: L0104
// Arrays and slices have no drain method.
fn main() {
    let a = [1, 2, 3];
    let _ = a.drain(..);
}
