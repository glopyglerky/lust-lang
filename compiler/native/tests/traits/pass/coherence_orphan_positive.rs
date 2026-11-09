// ARCH 2.6, orphan-based impossibility, positive case: A and My
// are local, My does not implement A, and no downstream crate can
// add that impl, so the blanket clause cannot hold for My.
// Rust accepts the pair. From the round-2 probe coh_ok.rs.
trait A {}
trait Tr {
    fn v(&self) -> u8;
}
impl<T: A> Tr for T {
    fn v(&self) -> u8 {
        1
    }
}
struct My;
impl Tr for My {
    fn v(&self) -> u8 {
        2
    }
}
fn main() {
    println!("{}", My.v());
}
