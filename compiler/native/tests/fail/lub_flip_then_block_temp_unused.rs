// expect: L0306
// rustc E0716: the LUB reborrow of the then block reads its dropped
// tail temporary even when the `if` value is unused.
static G: u8 = 9;
fn main() {
    let v = vec![1u8, 2];
    let k = v.len();
    if k > 0 { &&G } else { &v[0] };
}
