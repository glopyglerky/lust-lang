// expect: L0809
// A block with an item always breaks over lines in rustc's assert! text.
fn main() {
    assert!({
        fn f() -> bool {
            true
        }
        f()
    });
    println!("ok");
}
