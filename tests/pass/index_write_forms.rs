// Index writes that rustc accepts: the index is a plain local, the right side reads the
// Vec, two-phase method calls read the receiver, and writes go through nested places.
struct Bag {
    items: Vec<i32>,
}

fn main() {
    let mut v = vec![1, 2, 3];
    v[0] = v[1];
    v[0] += v[2];
    let i = v.len() - 1;
    v[i] = 9;
    v.push(v[0]);
    let n = v.len() as i32;
    v.push(n);
    let r = &mut v[i];
    *r += 1;
    println!("{:?}", v);
    let mut bag = Bag { items: vec![7] };
    bag.items.push(bag.items.len() as i32);
    bag.items[0] += 1;
    println!("{:?}", bag.items);
    let mut nested = vec![vec![5]];
    nested[0].push(6);
    println!("{:?}", nested);
}
