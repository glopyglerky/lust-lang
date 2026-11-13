#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct Unused {
    v: Vec<u8>,
}
#[derive(Clone, Debug)]
struct W<T> {
    t: T,
}
fn get<T: Clone>(w: &W<T>) -> T {
    w.t.clone()
}
fn main() {
    let w = W { t: String::from("w") };
    let u = Unused { v: vec![1] };
    println!("{} {}", get(&w.clone()), u.v.len());
    println!("{:?}", w);
}
