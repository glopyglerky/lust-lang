// expect: L0631
// rustc E0015: a non-const std fn in a const initializer is an error, also in a dead branch.
const C: u32 = if true { 1 } else { std::process::exit(1) };
fn main() {
    println!("{}", C);
}
