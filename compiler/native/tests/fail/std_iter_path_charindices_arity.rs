// expect: L0110
// rustc E0107: `CharIndices` takes a lifetime argument only.
fn type_only<T>() {}

fn main() {
    type_only::<std::str::CharIndices<u8>>();
}
