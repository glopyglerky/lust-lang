// expect: L0110
fn main() {
    let x @ 0..=254 = 3u8;
    println!("{}", x);
}
