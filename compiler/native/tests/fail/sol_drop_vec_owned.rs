// expect: L0306
// Sol review probe `drop_vec_owned`; rustc: E0597 `x` does not live long enough
struct Watch<'a>(&'a i32);
impl Drop for Watch<'_> { fn drop(&mut self) { println!("{}", self.0); } }
fn main() {
    let values;
    { let x = 1; values = vec![Watch(&x)]; }
}
