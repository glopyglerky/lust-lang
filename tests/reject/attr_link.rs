// expect: L0522
// Valid Rust; lust does not link native libraries, so the
// attribute stays a clean reject rather than a silent accept.
#[link(name = "c")]
fn f() {}

fn main() {
    f();
    println!("x");
}
