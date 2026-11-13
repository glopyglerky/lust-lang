// The errno module needs no feature.
fn main() {
    println!("{}", nix::errno::Errno::ENOENT as i32);
}
