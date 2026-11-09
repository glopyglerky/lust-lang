struct Unused {
    v: Vec<u8>,
}
struct W<T> {
    t: T,
}
fn get<T: Clone>(w: &W<T>) -> T {
    w.t.clone()
}
fn main() {
    let w = W { t: String::from("w") };
    let u = Unused { v: vec![1] };
    println!("{} {}", get(&w), u.v.len());
}
