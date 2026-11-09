// A retain predicate that captures `&self` over elements holding an `Arc<dyn Fn + Send + Sync>`
// (ts-rust goport_watch.rs:308): `&T` of an `Arc` cannot store the capture.
use std::sync::{Arc, Mutex};
type Cb = Arc<dyn Fn(&str) + Send + Sync>;
struct Watches { live: Mutex<Vec<(u64, String, bool, Cb)>> }
struct TestWatch { id: u64, watches: Arc<Watches> }
trait Watch { fn close(&self) -> Result<(), String>; }
impl Watch for TestWatch {
    fn close(&self) -> Result<(), String> {
        self.watches.live.lock().expect("watches").retain(|(id, _, _, _)| *id != self.id);
        Ok(())
    }
}
fn main() {
    let cb: Cb = Arc::new(|s: &str| println!("cb {s}"));
    let w = Arc::new(Watches { live: Mutex::new(vec![(1, "a".into(), true, cb.clone()), (2, "b".into(), false, cb)]) });
    let t = TestWatch { id: 1, watches: w.clone() };
    t.close().unwrap();
    let l = w.live.lock().unwrap();
    println!("{} {}", l.len(), l[0].0);
    (l[0].3)("x");
}
