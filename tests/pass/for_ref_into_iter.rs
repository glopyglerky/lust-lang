// `for x in &w` and `for x in &mut w` through user IntoIterator impls for
// `&W` and `&mut W` (W-STD-B), with builtin and user iterator types.
struct W {
    items: Vec<u8>,
}

impl<'a> IntoIterator for &'a W {
    type Item = &'a u8;
    type IntoIter = std::slice::Iter<'a, u8>;
    fn into_iter(self) -> std::slice::Iter<'a, u8> {
        self.items.iter()
    }
}

impl<'a> IntoIterator for &'a mut W {
    type Item = &'a mut u8;
    type IntoIter = std::slice::IterMut<'a, u8>;
    fn into_iter(self) -> std::slice::IterMut<'a, u8> {
        self.items.iter_mut()
    }
}

struct Owned {
    items: Vec<u8>,
}
impl<'a> IntoIterator for &'a Owned {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<u8>;
    fn into_iter(self) -> std::vec::IntoIter<u8> {
        self.items.clone().into_iter()
    }
}
fn main() {
    let o = Owned { items: vec![4, 5] };
    let mut t = 0u8;
    for x in &o {
        t += x;
    }
    println!("{t}");
    let mut w = W { items: vec![1, 2] };
    for x in &mut w {
        *x += 1;
    }
    let mut s = 0u8;
    for x in &w {
        s += *x;
    }
    println!("{:?} {}", w.items, s);
}
