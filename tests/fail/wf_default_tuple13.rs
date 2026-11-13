// expect: L0612
// core's `Default` for tuples stops at 12 elements, so a `V: Default` bound fails on a 13-tuple in an array.
struct Store<V: Default> {
    items: Vec<V>,
}

struct Checker {
    links: Store<[(u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8); 2]>,
}

fn main() {
    let c = Checker { links: Store { items: Vec::new() } };
    println!("{}", c.links.items.len());
}
