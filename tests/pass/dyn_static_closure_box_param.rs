// A closure param `&[Box<dyn Watch>]` (Box is covariant) does not stop a `'static` object use
// elsewhere in the body (ts_goport fswatch/watcher.rs:498).
use std::sync::Arc;
trait Watch: Send + Sync {
    fn close(&self) -> u32;
}
trait Opt {}
struct W(u32);
impl Watch for W {
    fn close(&self) -> u32 {
        self.0
    }
}
type Cb = Arc<dyn Fn(u32) + Send + Sync>;
struct Req<'a> {
    dir: String,
    cb: Cb,
    options: &'a [Box<dyn Opt>],
}
trait Watcher {
    fn watch(&self, dir: &str, cb: Cb, o: &[Box<dyn Opt>]) -> Result<Box<dyn Watch>, String>;
}
struct Fw;
impl Watcher for Fw {
    fn watch(&self, dir: &str, _cb: Cb, _o: &[Box<dyn Opt>]) -> Result<Box<dyn Watch>, String> {
        Ok(Box::new(W(dir.len() as u32)))
    }
}
struct F {
    primary: Arc<dyn Watcher>,
}
impl F {
    fn watch_all(&self, requests: &[Req<'_>]) -> Result<Vec<Box<dyn Watch>>, String> {
        let mut watches: Vec<Box<dyn Watch>> = Vec::with_capacity(requests.len());
        let rollback = |watches: &[Box<dyn Watch>]| {
            for w in watches.iter().rev() {
                let _ = w.close();
            }
        };
        for r in requests {
            let result = self.primary.watch(&r.dir, r.cb.clone(), r.options);
            match result {
                Ok(w) => watches.push(w),
                Err(e) => {
                    rollback(&watches);
                    return Err(e);
                }
            }
        }
        Ok(watches)
    }
}
fn main() {
    let f = F { primary: Arc::new(Fw) };
    let cb: Cb = Arc::new(|_| {});
    let reqs = vec![Req { dir: "ab".to_string(), cb: cb.clone(), options: &[] }, Req { dir: "abc".to_string(), cb, options: &[] }];
    let ws = f.watch_all(&reqs).unwrap();
    println!("{}", ws.iter().map(|w| w.close()).sum::<u32>());
}
