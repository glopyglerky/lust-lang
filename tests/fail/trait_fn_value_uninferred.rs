// expect: L0610
// Nothing fixes Self or the target of this `Into::into` (rustc E0790).
fn main() {
    let f = Into::into;
    let _ = f;
    println!("done");
}
