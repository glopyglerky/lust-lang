// expect: L0511
fn main() {
    pub(super) fn f() -> u32 {
        1
    }
    println!("{}", f());
}
