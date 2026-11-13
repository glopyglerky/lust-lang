// A generic Adt applied to a dyn type with and without its default object lifetime
// (`M<Rc<dyn Tr>>` in a body and in a signature) is one
// Rust type and needs one C type (tf-build: two instances that differ only in lifetimes
// got two C structs, "cannot convert 'struct A66' to 'struct A67'"; ts_goport's
// overlay_file_handles). Also with a Drop impl, derives, and `&'static` fields.
use std::fmt::Debug;
use std::rc::Rc;

trait Fh: Debug {
    fn n(&self) -> u8;
}

#[derive(Debug)]
struct O(u8);

impl Fh for O {
    fn n(&self) -> u8 {
        self.0
    }
}

#[derive(Debug)]
struct M<V> {
    v: Vec<V>,
}

impl<V> M<V> {
    fn new() -> Self {
        M { v: Vec::new() }
    }
    fn insert(&mut self, v: V) {
        self.v.push(v);
    }
}

struct Loud<T: Debug>(T);

impl<T: Debug> Drop for Loud<T> {
    fn drop(&mut self) {
        println!("drop {:?}", self.0);
    }
}

fn handles(src: &M<Rc<O>>) -> M<Rc<dyn Fh>> {
    let mut files: M<Rc<dyn Fh>> = M::new();
    for o in &src.v {
        files.insert(o.clone());
    }
    files
}

fn statics() -> M<Rc<dyn Fh>> {
    let mut m = M::new();
    m.insert(Rc::new(O(9)) as Rc<dyn Fh>);
    m
}

fn loud() -> Loud<M<&'static str>> {
    let mut m: M<&str> = M::new();
    m.insert("x");
    Loud(m)
}

fn main() {
    let mut src = M::new();
    src.insert(Rc::new(O(7)));
    let h = handles(&src);
    let s = statics();
    println!("{} {} {:?}", h.v[0].n(), s.v[0].n(), h);
    let both = vec![handles(&src), statics()];
    println!("{}", both.iter().map(|m| m.v.len()).sum::<usize>());
    let l = loud();
    let o: Option<Loud<M<&str>>> = Some(l);
    println!("{}", o.is_some());
}
