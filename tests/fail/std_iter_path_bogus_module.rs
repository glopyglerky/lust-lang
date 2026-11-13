// expect: L0102
// rustc E0425: there is no `Iter` in `std::sync` (it is in `std::sync::mpsc`).
fn type_only<T>() {}

fn main() {
    type_only::<std::sync::Iter<std::vec::IntoIter<u8>>>();
}
