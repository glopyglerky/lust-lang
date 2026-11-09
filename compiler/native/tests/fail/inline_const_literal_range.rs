// expect: L0630
// An unpinned inline const literal falls back to i32.
fn main() {
    let v = const { 9223372036854775807 };
    println!("{}", v);
}
