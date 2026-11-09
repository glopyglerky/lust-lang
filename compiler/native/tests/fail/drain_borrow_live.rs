// expect: L0303
// The Vec cannot be used while its Drain lives.
fn main() {
    let mut v = vec![1, 2, 3, 4];
    let d = v.drain(..2);
    v.push(5);
    println!("{}", d.count());
}
