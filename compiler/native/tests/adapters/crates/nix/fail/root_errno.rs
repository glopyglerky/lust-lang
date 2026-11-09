// expect: L0510
// rustc E0412: upstream imports `Errno` privately at the root; the public
// names are `nix::errno::Errno` and `nix::Error`.
fn f(e: nix::Errno) -> i32 {
    e as i32
}

fn main() {
    println!("{}", f(nix::errno::Errno::EPERM));
}
