// expect: L0641
// rustc E0599: Box::new_uninit needs T: Sized.
fn main() {
    let b = Box::<[u8]>::new_uninit();
    drop(b);
}
