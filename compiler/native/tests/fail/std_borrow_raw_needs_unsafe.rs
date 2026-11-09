// expect: L0404
// rustc E0133: `BorrowedFd::borrow_raw` is an `unsafe fn` in std.
fn main() {
    let fd = std::os::fd::BorrowedFd::borrow_raw(1);
    println!("{fd:?}");
}
