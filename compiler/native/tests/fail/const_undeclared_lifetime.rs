// expect: L0203
// rustc E0261: a const's type takes only `'static` or `'_`.
const C: Option<&'a str> = None;
fn main() {
    println!("{:?}", C);
}
