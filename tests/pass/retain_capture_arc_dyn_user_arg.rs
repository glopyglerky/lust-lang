// `retain` whose predicate captures `&self`, over elements holding an
// `Arc<dyn Fn(Ev)>` whose argument is a plain user struct (ts-rust
// goport_watch.rs:308): `Ev` holds no borrow, so the capture cannot reach
// the object's 'static lifetime. lust rejected it (L0733).
use std::sync::{Arc, Mutex};

struct Ev {
    p: u32,
}

type Cb = Arc<dyn Fn(Ev) + Send + Sync>;

struct Watches {
    live: Mutex<Vec<(u64, Cb)>>,
}

struct TestWatch {
    id: u64,
    watches: Arc<Watches>,
}

impl TestWatch {
    fn close(&self) {
        self.watches.live.lock().expect("w").retain(|(id, _)| *id != self.id);
    }
}

#[derive(Debug)]
struct Plain {
    n: u32,
}

fn main() {
    let cb: Cb = Arc::new(|v: Ev| println!("cb {}", v.p));
    let w = Arc::new(Watches { live: Mutex::new(vec![(1, cb.clone()), (2, cb)]) });
    TestWatch { id: 1, watches: w.clone() }.close();
    let l = w.live.lock().unwrap();
    println!("{} {}", l.len(), l[0].0);
    (l[0].1)(Ev { p: 5 });
    let mut v = vec![Plain { n: 1 }, Plain { n: 2 }, Plain { n: 3 }];
    let lim = String::from("ab");
    v.retain(|p| p.n as usize != lim.len());
    println!("{:?}", v);
}
