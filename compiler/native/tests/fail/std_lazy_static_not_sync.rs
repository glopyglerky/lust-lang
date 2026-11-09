// expect: L0612
// rustc E0277: `Rc<u8>` is not `Sync`, so the static's `LazyLock` is not.
use std::rc::Rc;
use std::sync::LazyLock;
static X: LazyLock<Rc<u8>> = LazyLock::new(|| Rc::new(1));
fn main() {
    println!("{}", **X);
}
