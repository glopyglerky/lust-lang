// expect: L0111
// rustc E0277: no From<{integer}> for String.
fn main() {
    let s: String = 5.into();
    println!("{s}");
}
