// expect: L0110
// rustc E0530: match bindings cannot shadow tuple structs (`Z` names `struct Z();`).
struct Z();
fn main() {
    let z = Z();
    match z {
        Z => {}
    }
}
