// expect: L0107
// rustc E0107: upstream `io::read` takes two generic parameters.
use rustix::fd::BorrowedFd;

fn main() {
    let mut b = [0u8; 0];
    let n = rustix::io::read::<BorrowedFd<'static>>(rustix::stdio::stdin(), &mut b);
    println!("{:?}", n.is_ok());
}
