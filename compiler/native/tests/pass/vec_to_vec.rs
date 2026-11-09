// [T]::to_vec through a Vec receiver (&Vec, Arc<Vec>): a clone of each element, exact capacity.
use std::sync::Arc;
#[derive(Clone, Debug)] struct N(String);
impl Drop for N { fn drop(&mut self) { println!("drop {}", self.0); } }
fn main() { let v = vec![N("a".into()), N("b".into())]; let w = v.to_vec(); let r = &v; let x = r.to_vec(); let a: Arc<Vec<N>> = Arc::new(vec![N("c".into())]); let y = a.to_vec(); let e: Vec<u8> = Vec::new(); println!("{:?} {:?} {:?} {} {}", w, x, y, e.to_vec().capacity(), w.capacity()); }
