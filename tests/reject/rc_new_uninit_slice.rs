// expect: L0104
// Valid Rust; lust leaves out Rc/Arc::new_uninit_slice: std panics at its own Layout::array sites for huge lengths.
fn main() {
    let r = std::rc::Rc::<[u8]>::new_uninit_slice(3);
    println!("{}", r.len());
}
