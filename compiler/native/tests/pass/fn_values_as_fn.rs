// Fn items, fn pointers, and tuple constructors implement the Fn
// traits: through a generic bound and as iterator adapter arguments.
#[derive(Debug)]
struct W(u8);

fn double(x: &u8) -> u8 {
    x * 2
}

fn add_one(x: u8) -> u8 {
    x + 1
}

fn keep(x: &&u8) -> bool {
    **x > 1
}

fn apply<F: Fn(u8) -> u8>(f: F, x: u8) -> u8 {
    f(x)
}

fn apply_mut<F: FnMut(u8) -> u8>(mut f: F, x: u8) -> u8 {
    let y = f(x);
    f(y)
}

fn main() {
    let v: Vec<u8> = vec![1, 2].iter().map(double).collect();
    let p: fn(&u8) -> u8 = double;
    let w: Vec<u8> = vec![3, 4].iter().map(p).collect();
    let ws: Vec<W> = vec![5u8, 6].into_iter().map(W).collect();
    let k: Vec<&u8> = v.iter().filter(keep).collect();
    let q: fn(u8) -> u8 = add_one;
    println!("{:?} {:?} {:?} {:?}", v, w, ws, k);
    println!("{} {} {}", apply(add_one, 1), apply(q, 2), apply_mut(add_one, 3));
}
