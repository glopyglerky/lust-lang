// expect: L0631
// rustc E0015: a call to a non-const fn in an anonymous const is an error, also in a dead branch.
fn nc() -> usize {
    1
}
struct Q<T>(T);
impl Q<[u8; {
    if false {
        nc();
    }
    3
}]> {
    fn n() -> usize {
        3
    }
}
fn main() {
    println!("{}", Q::<[u8; 3]>::n());
}
