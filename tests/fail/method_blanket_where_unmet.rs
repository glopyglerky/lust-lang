// expect: L0104
// rustc E0599: the blanket impl's where clause fails and no other
// candidate exists.
trait Tag {}
trait V {
    fn name(&self) -> u32;
}
impl<T: Tag> V for T {
    fn name(&self) -> u32 {
        1
    }
}
struct C;
fn main() {
    println!("{}", C.name());
}
