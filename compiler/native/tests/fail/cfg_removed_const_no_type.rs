// expect: L0001
// rustc: a const item with no type is a grammar error, also in cfg-removed code (Sol review 3, L14).
#[cfg(any())]
const C = 1;
fn main() {
    println!("ok");
}
