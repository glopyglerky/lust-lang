// expect: L0612
// rustc E0277: Stdio has no From<UnixStream> (only through OwnedFd).
use std::os::unix::net::UnixStream;
fn main() { let (a, _b) = UnixStream::pair().unwrap(); let _ = std::process::Stdio::from(a); }
