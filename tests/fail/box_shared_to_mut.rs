// expect: L0110
fn take_mut(x: &mut i32) {
    *x += 1;
}
fn main() {
    let b = Box::new(1);
    take_mut(&b);
    println!("{}", b);
}
