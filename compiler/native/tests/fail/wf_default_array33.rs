// expect: L0612
// core's `Default` for arrays stops at 32 elements, so a `V: Default` bound fails on `[u8; 33]`.
struct Store<V: Default> {
    items: Vec<V>,
}

struct Checker {
    links: Store<[u8; 33]>,
}

fn main() {
    let c = Checker { links: Store { items: Vec::new() } };
    println!("{}", c.links.items.len());
}
