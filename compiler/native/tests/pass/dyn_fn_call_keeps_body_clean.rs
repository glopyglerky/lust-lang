// A call through a `dyn Fn` in a body that also calls an `impl dyn Tr`
// method on a `'static` box: the call's output holds no dyn value, so
// the `'static` use stays provable (it was L0705).
use std::rc::Rc;
trait Fs { fn read(&self, p: &str) -> Option<String>; fn name(&self) -> String { String::from("fs") } fn generic<T>(&self, t: T) -> T where Self: Sized { t } }
struct Mem { files: Vec<(String, String)> }
impl Fs for Mem { fn read(&self, p: &str) -> Option<String> { match self.files.iter().find(|(k, _)| k == p) { Some((_, v)) => Some(v.clone()), None => None } } }
impl dyn Fs { fn read_or(&self, p: &str, d: &str) -> String { match self.read(p) { Some(v) => v, None => d.to_string() } } }
struct Host { fs: Box<dyn Fs>, handlers: Vec<(String, Box<dyn Fn(&str) -> usize>)> }
fn walk(fs: &dyn Fs, paths: &[&str], depth: u32) -> usize { if depth == 0 { return 0; } paths.iter().filter(|p| fs.read(p).is_some()).count() + walk(fs, paths, depth - 1) }
fn main() {
    let files = vec![(String::from("/a"), String::from("A"))];
    let host = Host { fs: Box::new(Mem { files }), handlers: Vec::new() };
    println!("{} {}", host.fs.read_or("/a", "-"), host.fs.read_or("/b", "-"));
    println!("{} {}", walk(&*host.fs, &["/a", "/b"], 3), host.fs.name());
    let prefix = Rc::new(String::from(">"));
    let p2 = prefix.clone();
    let mut h = host;
    h.handlers.push((String::from("len"), Box::new(move |s: &str| { p2.len() + s.len() })));
    if let Some((_, f)) = h.handlers.iter().find(|(n, _)| n == "len") { println!("{}", f("hello")); }
    println!("{}", Rc::strong_count(&prefix));
}
