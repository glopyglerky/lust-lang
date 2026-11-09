// expect: L0110
// rustc E0308: a `let` initializer of another type in a const.
const X: u32 = {
    let _a: u8 = 1u16;
    5
};
fn main() {
    println!("{}", X);
}
