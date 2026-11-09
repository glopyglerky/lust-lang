// expect: L0306
// rustc E0716: the else block's value is coerced after the block even
// when the `if` value is unused.
fn main() {
    let s = String::from("ab");
    let k = s.len();
    if k > 0 { s.as_str() } else { &&s[..] };
}
