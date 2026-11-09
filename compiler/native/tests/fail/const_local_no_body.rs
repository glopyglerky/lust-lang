// expect: L0001
// rustc: free constant item without body, also inside a fn body.
fn f() {
    const X: u8;
}
fn main() {}
