// expect: L0802
// rustc E0433: a leading `::` names an extern crate; `println` is a std macro, not a crate.
fn main() {
    ::println!("a");
}
