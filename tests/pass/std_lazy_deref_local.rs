// A local `LazyLock` (not a static): method calls through its `Deref`.
use std::sync::LazyLock;
fn main() {
    let l = LazyLock::new(|| vec![1, 2, 3]);
    println!("{} {}", l.len(), l.contains(&2));
}
