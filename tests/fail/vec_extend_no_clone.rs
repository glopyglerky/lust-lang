// expect: L0112
struct NoClone {
    n: i32,
}
fn main() {
    let src = [NoClone { n: 1 }];
    let mut v: Vec<NoClone> = Vec::new();
    v.extend_from_slice(&src);
    println!("{}", v.len());
}
