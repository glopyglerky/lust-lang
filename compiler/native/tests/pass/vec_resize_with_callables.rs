// resize_with takes closures and fn pointers; the closure runs once per new element (R4).
#[derive(Debug, Default, Clone)]
struct C { x: u8 }
fn main() {
    let mut t: Vec<Option<u8>> = vec![]; t.resize_with(2, || None); println!("{t:?}");

    let f: fn() -> u8 = || 3; let mut b = vec![1u8]; b.resize_with(3, f); println!("{b:?}");
}
