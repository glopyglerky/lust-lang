// expect: L0203
// Valid Rust: `T: 'a` on an impl's own params is not supported yet (only fns
// and inherent methods take it).
struct W<'a, T>(&'a T);
impl<'a, T: 'a> W<'a, T> {
    fn get(&self) -> &'a T {
        self.0
    }
}
fn main() {
    let x = 5;
    println!("{}", W(&x).get());
}
