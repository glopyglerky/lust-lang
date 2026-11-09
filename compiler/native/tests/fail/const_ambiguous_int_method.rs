// expect: L0111
// rustc E0689: no method calls on an ambiguous integer type, in a
// const initializer too.
const C: usize = 7 + 1.wrapping_mul(3);

fn main() {
    println!("{}", C);
}
