// `io::Write::write` for `Vec<u8>` is not in scope without a `use`, so
// rustc's probe takes the user trait's `write` (check/std_inherent.rs
// counts a non-prelude std trait only where it may be imported).
trait Sink {
    fn write(&mut self, b: u8) -> usize;
}
impl Sink for Vec<u8> {
    fn write(&mut self, b: u8) -> usize {
        self.push(b);
        self.len()
    }
}
fn main() {
    let mut v: Vec<u8> = Vec::new();
    println!("{}", v.write(4));
    println!("{}", v.write(5));
}
