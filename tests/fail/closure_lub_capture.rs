// expect: L0110
// rustc E0308: a capturing closure does not join the other branch's closure.
fn main() {
    let c = std::env::args().count() > 5;
    let k = 3u8;
    let f = if c { |x: u8| x + 1 } else { |x: u8| x + k };
    println!("{}", f(3));
}
