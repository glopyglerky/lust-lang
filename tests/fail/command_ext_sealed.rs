// expect: L0612
// rustc E0277: std's `CommandExt` is sealed (its supertrait is private), so a
// user type cannot implement it.
struct S;
impl std::os::unix::process::CommandExt for S {
    fn exec(&mut self) -> std::io::Error {
        std::io::Error::other("no")
    }
}
fn main() {}
