// expect: L0612
// rustc E0277: upstream has no `NixPath` impl for `OsString`.
use nix::NixPath;

fn main() {
    let p = std::ffi::OsString::from("/tmp");
    println!("{}", NixPath::len(&p));
}
