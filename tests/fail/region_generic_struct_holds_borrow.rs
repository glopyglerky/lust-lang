// expect: L0312
// rustc E0506: a generic struct holding `&v` keeps `v` borrowed while it is live.
struct W<T> {
    x: T,
}

fn main() {
    let mut v = 5;
    let w = W { x: &v };
    v = 6;
    println!("{} {}", w.x, v);
}
