// expect: L0612
// rustc E0277: upstream `accessat<P: Arg, Fd: AsFd>` takes the path type first.
use rustix::fd::BorrowedFd;
use rustix::fs::{Access, AtFlags, CWD, accessat};

fn main() {
    let r = accessat::<BorrowedFd<'static>, &str>(CWD, "/", Access::EXISTS, AtFlags::empty());
    println!("{}", r.is_ok());
}
