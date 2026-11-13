// A method of `impl<'a, I: Iterator<Item = &'a u32>> W<I>` reads the impl's `Item` bound
// (tf-borrow gen54).
struct W<I> {
    it: I,
}
impl<'a, I: Iterator<Item = &'a u32>> W<I> {
    fn get(&mut self) -> Option<&'a u32> { self.it.next() }
}
fn main() {
    let n = [1u32, 2];
    let mut v = Vec::new();
    {
        let mut w = W { it: n.iter() };
        if let Some(x) = w.get() { v.push(x); }
    }
    println!("{}", v.len());
}
