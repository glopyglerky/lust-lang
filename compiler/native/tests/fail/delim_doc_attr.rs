// expect: L0001
// rustc rejects a `doc` attribute whose `[` closes with `)`; the `(` in the string does not count.
#[doc = "a ( b")
struct S;

fn main() {
    let _ = S;
}
