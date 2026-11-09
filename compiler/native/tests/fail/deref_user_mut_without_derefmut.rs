// expect: L0308
// rustc E0596: a `&mut self` method through a user `Deref` impl needs
// `DerefMut`.
use std::ops::Deref;
struct G {
    items: Vec<u32>,
}
impl Deref for G {
    type Target = [u32];
    fn deref(&self) -> &[u32] {
        &self.items
    }
}
fn main() {
    let mut g = G { items: vec![3, 1] };
    for x in g.iter_mut() {
        *x += 1;
    }
    println!("{}", g.len());
}
