// expect: L0106
// Two items of one name in one block (rustc E0428), also in a
// single-file program that skips the resolver.
fn main() {
    struct W;
    struct W;
}
