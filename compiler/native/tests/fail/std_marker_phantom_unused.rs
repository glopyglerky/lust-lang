// expect: L0110
// rustc E0392: a user unit struct with an unused param is still an error.
struct Ph<T>;
fn main() {
    let _p: Ph<u8> = Ph;
}
