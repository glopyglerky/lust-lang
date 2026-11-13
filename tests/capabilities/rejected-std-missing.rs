// capability: rejected: std and sysroot crate items lust does not have yet, by module path (named in the message)
// expect: L0505

fn main() {
    let c = std::ffi::CString::new("a").unwrap();
    println!("{:?}", c);
}
