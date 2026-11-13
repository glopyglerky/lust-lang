// Sol review probe `drop_moved_outer_local`: rustc accepts.
struct Watch<'a>(&'a i32);
impl Drop for Watch<'_> { fn drop(&mut self) { println!("{}", self.0); } }
fn main() {
    let value;
    { let x = 1; value = Watch(&x); drop(value); }
}
