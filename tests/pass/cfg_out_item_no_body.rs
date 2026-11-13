// A cfg-disabled item is stripped before rustc validates bodies,
// so a no-body const or static under #[cfg(any())] is accepted;
// enabled, both are errors (tests/fail/const_item_no_body.rs).
#[cfg(any())]
const X: u8;
#[cfg(any())]
static Y: u8;
fn main() {
    println!("ok");
}
