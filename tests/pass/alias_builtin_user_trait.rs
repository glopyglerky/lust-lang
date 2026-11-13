// `V4::mk()` through an alias of a builtin type calls the user trait's
// `mk` for `Vec<u8>`: std has no item named `mk` for Vec (L0643 rejects
// one that std has, see tests/fail/std_shadow_path_fn.rs).
trait Mk {
    fn mk() -> Self;
}
impl Mk for Vec<u8> {
    fn mk() -> Self {
        vec![1]
    }
}
type V4 = Vec<u8>;
fn main() {
    println!("{:?}", V4::mk());
}
