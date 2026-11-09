// expect: L0301
// rustc E0382: a move on a branch that never runs still counts.
struct S {
    x: u32,
}
static C: u32 = {
    let s = S { x: 1 };
    if 1 > 2 {
        let _t = s;
    }
    s.x
};
fn main() {
    println!("{}", C);
}
