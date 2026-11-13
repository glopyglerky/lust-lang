// expect: L0104
// rustc E0599: there is no `checked_div_ceil` in core (only
// `div_ceil`), so lust must not invent one.
fn main() {
    println!("{:?}", 9u32.checked_div_ceil(3));
}
