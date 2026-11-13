// A source known only by an `IntoIterator` bound converts in
// `from_iter` (and `extend`), through its `into_iter` call.
fn from_src<S: IntoIterator<Item = u32>>(s: S) -> Vec<u32> { Vec::from_iter(s) }
fn main() { println!("{:?}", from_src([3, 4])); }
