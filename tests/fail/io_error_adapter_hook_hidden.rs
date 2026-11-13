// expect: L0104
// rustc E0599: std's io::Error has no `__lust_adapter_error`; the hook is for lust's adapter crates only.
fn show(f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "e")
}

fn main() {
    let e = std::io::Error::__lust_adapter_error(std::io::ErrorKind::Other, show, show);
    println!("{:?}", e);
}
