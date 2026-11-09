// expect: L0104
// rustc E0599: upstream spells EACCES as `Errno::ACCESS`.
fn main() {
    println!("{}", rustix::io::Errno::ACCES.raw_os_error());
}
