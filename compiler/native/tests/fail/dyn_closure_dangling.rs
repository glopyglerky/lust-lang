// expect: L0306
// rustc E0597: the closure keeps a `&dyn Read` to `n` past its scope.
trait Read {
    fn read(&self) -> u32;
}
impl Read for u32 {
    fn read(&self) -> u32 {
        *self
    }
}
fn main() {
    let f;
    {
        let n = 7u32;
        let d: &dyn Read = &n;
        f = move || d.read();
    }
    println!("{}", f());
}
