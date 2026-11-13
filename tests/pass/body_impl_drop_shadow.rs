// Drop impls of a block Restore and a module Restore (ts-rust guard types).
struct Restore;
impl Drop for Restore { fn drop(&mut self) { println!("module restore"); } }
fn f() {
    struct Restore(u8);
    impl Drop for Restore { fn drop(&mut self) { println!("block restore {}", self.0); } }
    let _r = Restore(1);
}
fn main() { f(); let _m = Restore; }
