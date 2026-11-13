// expect: L0511
// rustc E0433: `pub(in self::x)` names no module `x`. A file with no other
// resolver keyword took lust's no-resolve path, which skipped the check.
fn main() {
    pub(in self::x) struct P;
    println!("a");
}
