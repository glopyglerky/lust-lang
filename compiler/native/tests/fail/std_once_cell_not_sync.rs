// expect: L0612
// rustc E0277: `OnceCell<u8>` is `Send` but not `Sync`.
use std::cell::OnceCell;
fn need_sync<T: Sync>(_: &T) {}
fn main() {
    let c: OnceCell<u8> = OnceCell::new();
    need_sync(&c);
}
