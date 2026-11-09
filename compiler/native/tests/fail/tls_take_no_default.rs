// expect: L0612
// `take` needs `T: Default` (E0277).
use std::cell::RefCell;
struct NoDefault(u8);
thread_local! {
    static S: RefCell<NoDefault> = RefCell::new(NoDefault(1));
}
fn main() {
    let _x = S.take();
}
