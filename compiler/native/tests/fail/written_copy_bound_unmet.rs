// expect: L0301
// rustc E0382: `impl<T: Copy> Copy for G<T>` does not apply to `G<String>`.
struct G<T>(T);
impl<T: Clone> Clone for G<T> {
    fn clone(&self) -> Self {
        G(self.0.clone())
    }
}
impl<T: Copy> Copy for G<T> {}
fn main() {
    let a = G(String::from("x"));
    let b = a;
    let c = a;
    println!("{} {}", b.0, c.0);
}
