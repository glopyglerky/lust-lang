// expect: L0001
// rustc: `const C: u8 = ;` is a grammar error, also in cfg-removed code (Sol review 3, L14).
#[cfg(any())]
const C: u8 = ;
fn main() {
    println!("ok");
}
