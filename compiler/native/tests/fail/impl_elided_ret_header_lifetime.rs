// expect: L0734
// rustc: `impl` item signature doesn't match `trait` item signature. The
// elided inner lifetime is `&self`'s, the trait's is the impl header's.
struct L;
impl AsRef<&L> for Box<L> {
    fn as_ref(&self) -> &&L {
        todo!()
    }
}
fn main() { println!("ok"); }
