// expect: L0612
// rustc E0277: `OnceCell<ND>: Debug` needs `ND: Debug`. The cell's
// type is only known after `from` is selected, so the format test runs
// at body end; its goal must still be proved (fx-splice-derive).
struct ND;
fn main() {
    println!("{:?}", std::cell::OnceCell::from(ND));
}
