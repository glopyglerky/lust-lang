// expect: L0734
// rustc: `impl` item signature doesn't match `trait` item signature.
struct It<'a> {
    v: &'a [u8],
    i: usize,
}
impl<'a> Iterator for It<'a> {
    type Item = &'a u8;
    fn next(&mut self) -> Option<&u8> {
        let r = self.v.get(self.i);
        self.i += 1;
        r
    }
}
fn main() {
    let v = [1u8];
    let mut it = It { v: &v, i: 0 };
    println!("{:?}", it.next());
}
