// expect: L0110
// rustc E0107: `vec::IntoIter` takes one type argument.
fn bad(v: Vec<u8>) -> std::vec::IntoIter<u8, u8> {
    v.into_iter()
}

fn main() {}
