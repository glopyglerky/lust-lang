// expect: L0110
// rustc E0391: the fn-body `A` refers to itself, not to the module `A`.
const A: u32 = 5;

fn main() {
    const A: u32 = A + 1;
    println!("{}", A);
}
