// expect: L0110
// rustc E0308: a const initializer's body checker error counts, also in a branch the evaluator skips (Sol review 3, L1).
const N: u32 = if true {
    1
} else {
    false
};
fn main() {
    println!("{}", N);
}
