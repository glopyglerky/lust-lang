// expect: L0110
// rustc E0308: a closure that captures `k` cannot become a fn pointer.
fn main() {
    let k = 3u8;
    let p: fn(u8) -> u8 = |x| x + k;
    println!("{}", p(4));
}
