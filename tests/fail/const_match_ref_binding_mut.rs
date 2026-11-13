// expect: L0630
// rustc (edition 2024): cannot mutably bind by value within an
// implicitly-borrowing pattern.
struct P(u32, u32);
const Z: u32 = {
    let P(mut a, _) = &P(1, 2);
    a
};
fn main() {
    println!("{}", Z);
}
