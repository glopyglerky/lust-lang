// expect: L0612
// rustc E0277: `I` is not an iterator without a bound.
fn add<T, I>(v: &mut Vec<T>, i: I) {
    v.extend(i);
}

fn main() {
    let mut v = vec![1];
    add(&mut v, [2]);
    println!("{:?}", v);
}
