// expect: L0001
// A combining mark (U+0301) cannot start a name.
fn main() {
    let ́x = 1;
    println!("{}", ́x);
}
