// expect: L0110
// rustc E0107: `vec::IntoIter` takes no lifetime argument.
fn bad(v: Vec<u8>) -> std::vec::IntoIter<'static, u8> {
    v.into_iter()
}

fn main() {}
