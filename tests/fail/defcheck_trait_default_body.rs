// expect: L0104
// rustc E0277: a never-called trait default body is checked
// at the trait definition.
trait Bad {
    fn f(&self) {
        let _: String = 1u8.into();
    }
}
fn main() {}
