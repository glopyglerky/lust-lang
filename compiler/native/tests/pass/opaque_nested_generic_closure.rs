// A generic fn's `impl Iterator` (it holds a closure of its instance)
// returned again by another fn's `impl Iterator`. The caller of the outer
// fn once kept the template's closure type, and codegen failed with
// "`{closure}` (TemplateClosure) has no generated body" (ts_goport
// `PrintTables::for_each_value_node`, fx-cgen-ice).
use std::collections::HashMap;

struct Pages<V> {
    pages: Vec<Option<Box<[Option<V>; 4]>>>,
}

impl<V> Pages<V> {
    fn values(&self) -> impl Iterator<Item = &V> {
        self.pages.iter().flatten().flat_map(|p| p.iter().flatten())
    }
}

struct Originals {
    synthetic: Pages<u32>,
    other: HashMap<u32, u32>,
}

impl Originals {
    fn values(&self) -> impl Iterator<Item = u32> {
        self.synthetic.values().chain(self.other.values()).copied()
    }

    fn for_each_value(&self, mut f: impl FnMut(u32)) {
        self.values().for_each(&mut f);
    }
}

fn inner<T: Copy>(x: T) -> impl Iterator<Item = T> {
    (0..2).map(move |_| x)
}

fn plain() -> impl Iterator<Item = u32> {
    inner(7u32)
}

fn generic<U: Copy>(u: U) -> impl Iterator<Item = U> {
    inner(u)
}

fn twice<U: Copy + std::fmt::Debug>(u: U) -> impl Iterator<Item = String> {
    generic(u).map(|v| format!("{v:?}"))
}

fn main() {
    let mut other = HashMap::new();
    other.insert(9, 10);
    let o = Originals { synthetic: Pages { pages: vec![Some(Box::new([Some(1), None, Some(2), None])), None] }, other };
    let mut roots = vec![0];
    o.for_each_value(|n| roots.push(n));
    println!("{roots:?}");

    for v in plain() {
        println!("plain {v}");
    }
    plain().for_each(|v| println!("each {v}"));
    println!("{:?}", generic('a').collect::<Vec<_>>());
    println!("{:?}", generic(1.5f64).map(|v| v + 1.0).collect::<Vec<_>>());
    println!("{:?}", twice(3u8).collect::<Vec<_>>());
    println!("{}", plain().map(|v| v * 2).sum::<u32>());
}
