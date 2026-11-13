use std::sync::Arc;
#[derive(Clone, Debug)]
enum FileContent { Text(Arc<str>), Bytes(Arc<[u8]>) }
fn main() { let f = FileContent::Text(Arc::from("x")); let g = FileContent::Bytes(Arc::from(&b"ab"[..])); if let (FileContent::Text(t), FileContent::Bytes(b)) = (f.clone(), g.clone()) { println!("{t} {:?}", b); } println!("{:?} {:?}", f, g); }
