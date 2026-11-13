// expect: L0540
fn main() {
    let s: u32 = 33;
    let r = 1u32 << s;
    println!("{}", r);
}
