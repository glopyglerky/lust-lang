// expect: L0306
// rustc E0597: a default body stores a local borrow where `'a` data goes.
trait Pick {
    fn bad<'a>(&self, v: &mut Vec<&'a u8>) {
        let z = 5u8;
        v.push(&z);
    }
}
fn main() {}
