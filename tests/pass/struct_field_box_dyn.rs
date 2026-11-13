// A struct field holding `Box<dyn Tr>` under a reference keeps its
// `'static` object lifetime in the field's region map (tsrust T13
// fswatch/watcher.rs `watch_directory`).
pub trait O {
    fn n(&self) -> u8;
}
struct K;
impl O for K {
    fn n(&self) -> u8 {
        1
    }
}
pub struct Req<'a> {
    pub dir: String,
    pub options: &'a [Box<dyn O>],
}
fn count(reqs: &[Req<'_>]) -> usize {
    reqs.iter().map(|r| r.options.len() + r.dir.len()).sum()
}
fn watch(dir: &str, opts: &[Box<dyn O>]) -> usize {
    count(&[Req { dir: dir.to_string(), options: opts }])
}
fn main() {
    let v: Vec<Box<dyn O>> = vec![Box::new(K)];
    println!("{} {}", watch("d", &v), v[0].n());
}
