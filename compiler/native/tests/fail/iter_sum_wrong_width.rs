// expect: L0612
fn main() {
    let v = vec![1u8, 2];
    let s: u32 = v.iter().sum();
    println!("{}", s);
}
