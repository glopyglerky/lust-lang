// expect: L0612
// rustc E0277: no `From<u16>` for `u8`, though the target comes later.
fn main() {
    let x = 700u16.into();
    let y: u8 = x;
    println!("{y}");
}
