// expect: L0733
// rustc: lifetime may not live long enough; a default body returns `'b` data as `'a`.
trait Pick {
    fn bad<'a, 'b>(&self, _x: &'a u8, y: &'b u8) -> &'a u8 {
        y
    }
}
fn main() {}
