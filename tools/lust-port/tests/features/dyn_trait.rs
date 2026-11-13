// feature: dyn.trait
// form: &dyn and &mut dyn calls through a user trait, as in ts-rust Fs and Conn traits
trait Fs {
    fn exists(&self, path: &str) -> bool;
    fn bump(&mut self) -> i32;
}
struct MemFs { n: i32 }
impl Fs for MemFs {
    fn exists(&self, path: &str) -> bool { path == "/a" }
    fn bump(&mut self) -> i32 { self.n += 1; self.n }
}
fn check(fs: &dyn Fs) -> bool { fs.exists("/a") }
fn bump(fs: &mut dyn Fs) -> i32 { fs.bump() }
fn main() {
    let mut fs = MemFs { n: 0 };
    println!("{} {}", check(&fs), bump(&mut fs));
}
