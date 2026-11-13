// `HashMap::get` and the map iterators' items hold the map loan and the
// map's V, not more: a `'static` value cloned out does not keep the map
// borrowed for `'static` (tsrust T13 fswatch/debounce.rs `fire_callbacks`).
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
struct L {
    callbacks: HashMap<usize, Arc<dyn Fn() + Send + Sync>>,
}
struct D {
    mu: Mutex<L>,
}
impl D {
    fn fire(&self) -> usize {
        let cbs: Vec<Arc<dyn Fn() + Send + Sync>> = {
            let d = self.mu.lock().unwrap();
            let mut cbs: Vec<Arc<dyn Fn() + Send + Sync>> = Vec::with_capacity(d.callbacks.len());
            for cb in d.callbacks.values() {
                cbs.push(cb.clone());
            }
            cbs
        };
        for cb in &cbs {
            cb();
        }
        cbs.len()
    }
    fn one(&self, k: usize) -> Option<Arc<dyn Fn() + Send + Sync>> {
        self.mu.lock().unwrap().callbacks.get(&k).cloned()
    }
}
fn first<'a>(m: &HashMap<u32, &'a str>) -> Option<&'a str> {
    m.values().next().copied()
}
fn main() {
    let mut m: HashMap<usize, Arc<dyn Fn() + Send + Sync>> = HashMap::new();
    m.insert(1, Arc::new(|| println!("cb")));
    let d = D { mu: Mutex::new(L { callbacks: m }) };
    println!("{} {}", d.fire(), d.one(1).is_some());
    let mut h = HashMap::new();
    h.insert(1, "x");
    let f = first(&h);
    h.insert(2, "y");
    println!("{:?}", f);
}
