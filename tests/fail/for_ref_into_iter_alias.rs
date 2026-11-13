// expect: L0303
// rustc E0499: the loop holds `&mut w` while the body borrows `w` again.
struct W {
    items: Vec<u8>,
}

impl<'a> IntoIterator for &'a W {
    type Item = &'a u8;
    type IntoIter = core::slice::Iter<'a, u8>;
    fn into_iter(self) -> core::slice::Iter<'a, u8> {
        self.items.iter()
    }
}

impl<'a> IntoIterator for &'a mut W {
    type Item = &'a mut u8;
    type IntoIter = core::slice::IterMut<'a, u8>;
    fn into_iter(self) -> core::slice::IterMut<'a, u8> {
        self.items.iter_mut()
    }
}
fn main() {
    let mut w = W { items: vec![1] };
    for x in &mut w {
        w.items.push(*x);
    }
}
