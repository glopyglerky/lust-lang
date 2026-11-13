// Edits inside a type a central struct holds by value (`Leaf` in `Hub`):
// the `&Hub` callers of `Hub::get` read `Hub` by name only; the bodies that
// open `Leaf`, hold a `Hub` by value (also inside a closure an opaque
// returns, and in an `Option`), or ask `Sync` of it see each edit.
#[derive(Default)]
struct Noisy;
impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop");
    }
}
#[derive(Default)]
struct Leaf {
    x: i64,
}
struct Hub {
    leaf: Leaf,
    n: i64,
}
impl Hub {
    fn new() -> Hub {
        Hub { leaf: Leaf { x: 1, ..Default::default() }, n: 2 }
    }
    fn get(&self) -> i64 {
        self.leaf.x + self.n
    }
}
fn need_sync<T: Sync>(_: &T) {}
fn q1(h: &Hub) -> i64 {
    h.get() + 1
}
fn q2(h: &Hub) -> i64 {
    h.get() * 2
}
fn q3(h: &Hub) -> i64 {
    q1(h) + q2(h)
}
fn later() -> impl Fn() -> i64 {
    let h = Hub::new();
    move || h.get()
}
fn main() {
    let h = Hub::new();
    need_sync(&h);
    let o: Option<Hub> = Some(Hub::new());
    let f = later();
    println!("{} {} {}", q3(&h), o.map_or(0, |h| h.get()), f());
}
