// expect: L0612
// rustc E0277: `&i32` is not `AsRef<OsStr>`, so no From impl of OsString takes it.
fn main() {
    let o = std::ffi::OsString::from(&5i32);
    println!("{:?}", o);
}
