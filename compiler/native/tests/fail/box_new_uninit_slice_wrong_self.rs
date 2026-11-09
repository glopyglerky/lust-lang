// expect: L0104
// rustc E0599: new_uninit_slice is on Box<[T]>, so the written Self is Box<[u8]>, not Box<u8>.
fn main() {
    let b = Box::<u8>::new_uninit_slice(2);
    println!("{}", b.len());
}
