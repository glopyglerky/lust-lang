// expect: L0306
// A Drain cannot outlive the Vec it borrows.
fn take(d: std::vec::Drain<'_, i32>) {}
fn main() {
    let d = {
        let mut v = vec![1, 2, 3];
        let d = v.drain(..);
        d
    };
    take(d);
}
