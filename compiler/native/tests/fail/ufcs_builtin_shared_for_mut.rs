// expect: L0110
// rustc E0308: `Vec::push` takes `&mut Vec<T>`, not `&Vec<T>`.
fn main() {
    let mut v = vec![1];
    Vec::push(&v, 2);
    v.push(3);
    println!("{:?}", v);
}
