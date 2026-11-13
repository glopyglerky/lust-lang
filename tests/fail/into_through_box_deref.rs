// expect: L0104
// rustc E0277: core's blanket `Into` is picked at the first autoderef step, so
// `Box<String>` needs `OsString: From<Box<String>>`; the box's `String` does not count.
fn main() {
    let v = Box::new(String::from("bx"));
    let o: std::ffi::OsString = v.into();
    println!("{:?}", o);
}
