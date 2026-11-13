// expect: L0630
// rustc E0597: `T: 'static` with `T` a borrow of a local.
const fn keep<T: 'static>(_t: T) -> u32 {
    0
}
const X: u32 = {
    let a = 1u8;
    keep(&a)
};
fn main() {
    println!("{}", X);
}
