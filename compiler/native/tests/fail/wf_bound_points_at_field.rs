// expect: L0612
struct Store<V: Default> {
    items: Vec<V>,
}
struct Bad {
    good: u8,
    bad: Store<[u8; 33]>,
}
fn main() {}
