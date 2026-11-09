// rustc-error: E0277
// lust-expect: L0612
// ARCH 2.7 table: &T is Send only when T is Sync (and an inference variable is never bound from the env).
fn need<T: Send>(_t: &T) {}
fn g<T: Send>(t: &T) { need(&t); }
fn main() { g(&5u8); }
