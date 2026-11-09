// `Option::from_iter` builds the try-collected `Option<C>` (F3); was tests/reject (L0640).
fn main() { let v: Option<Vec<u32>> = Option::from_iter([Some(1), Some(2)]); println!("{:?}", v); }
