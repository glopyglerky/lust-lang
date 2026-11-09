// expect: L0303
// rustc E0502: the shared reborrow through `&m` is alive while `m` writes.
fn shared<'a, 'b>(x: &'a &'b mut Vec<i32>) -> &'a Vec<i32> {
    x
}
fn main() {
    let mut v = vec![1];
    let mut m = &mut v;
    let r = shared(&m);
    m.push(2);
    println!("{:?}", r);
}
