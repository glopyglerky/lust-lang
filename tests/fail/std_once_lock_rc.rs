// expect: L0612
// rustc E0277: `OnceLock<Rc<u8>>` is not `Sync`.
use std::sync::OnceLock;
fn need_sync<T: Sync>(_: &T) {}
fn main() {
    let o: OnceLock<std::rc::Rc<u8>> = OnceLock::new();
    need_sync(&o);
}
