#[derive(Clone, Debug, PartialEq)]
struct A {
    s: String,
}
fn dup<T: Clone>(t: &T) -> T {
    t.clone()
}
fn main() {
    let o = Some(A { s: String::from("a") });
    let t = (A { s: String::from("b") }, 7u8);
    let o2 = dup(&o);
    let t2 = t.clone();
    println!("{:?} {:?} {}", o2, t2.0, o == o2);
}
