// expect: L0112
// rustc E0369: the field type's generic `PartialEq` impl needs
// `T: PartialEq`, which `NoEq` does not have.
struct NoEq(u8);
struct M<T> {
    v: Vec<T>,
}
impl<T: PartialEq> PartialEq for M<T> {
    fn eq(&self, o: &M<T>) -> bool {
        self.v == o.v
    }
}
#[derive(PartialEq)]
struct W {
    m: M<NoEq>,
}
fn main() {
    println!("x");
}
