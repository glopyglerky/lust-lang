// expect: L0110
// rustc E0271: type mismatch resolving `<[&str; 1] as IntoIterator>::Item == i32`.
fn add<T, I: IntoIterator<Item = T>>(v: &mut Vec<T>, i: I) {
    v.extend(i);
}

fn main() {
    let mut v: Vec<i32> = vec![];
    add(&mut v, ["x"]);
    println!("{:?}", v);
}
