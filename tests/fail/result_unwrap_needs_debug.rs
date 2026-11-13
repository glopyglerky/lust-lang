// expect: L0612
// rustc E0277: Result::unwrap needs `E: Debug` for its panic message.
struct NoDebug;
fn main() {
    let r: Result<u8, NoDebug> = Ok(1);
    let _ = r.unwrap();
}
