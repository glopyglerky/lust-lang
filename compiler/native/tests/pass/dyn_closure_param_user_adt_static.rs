// A closure param annotation `Rc<dyn FileHandle>` fixes a type param of a user generic Adt;
// the return type's `'static` default reaches it later (ts-rust project/parsecache.rs). rustc
// infers the annotation's object lifetime, and both instances share one C type.
use std::rc::Rc;
trait FileContent { fn content(&self) -> String; }
trait FileHandle: FileContent { fn version(&self) -> i32; }
struct F;
impl FileContent for F { fn content(&self) -> String { "c".into() } }
impl FileHandle for F { fn version(&self) -> i32 { 1 } }
struct Cache<K, V, A> { parse: Box<dyn Fn(&K, A) -> V> }
fn new_cache<K, V, A>(parse: impl Fn(&K, A) -> V + 'static) -> Rc<Cache<K, V, A>> { Rc::new(Cache { parse: Box::new(parse) }) }
type PC = Cache<u32, String, Rc<dyn FileHandle>>;
fn new_pc() -> Rc<PC> {
    new_cache(|key: &u32, fh: Rc<dyn FileHandle>| -> String {
        let c = fh.content();
        format!("{key}{c}{}", fh.version())
    })
}
struct Own<X>(X);
trait P { fn p(&self) -> u32; }
impl P for u32 { fn p(&self) -> u32 { *self } }
fn take(o: Own<Box<dyn P>>) -> u32 { o.0.p() }
fn mk() -> Own<Box<dyn P>> { let c = |o: Own<Box<dyn P>>| o; c(Own(Box::new(3u32))) }
fn main() {
    let c = new_pc();
    println!("{}", (c.parse)(&3, Rc::new(F)));
    let o: Own<Box<dyn P>> = Own(Box::new(4u32));
    println!("{} {}", take(o), take(mk()));
}
