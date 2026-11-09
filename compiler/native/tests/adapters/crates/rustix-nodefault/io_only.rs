// The io module needs no feature.
fn main() {
    println!("{}", rustix::io::Errno::NOENT.raw_os_error());
}
