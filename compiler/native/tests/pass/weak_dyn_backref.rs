// The ts-rust fswatch shape: a backend keeps a `Weak<dyn WatcherImpl>`
// back-reference in a `OnceLock`, set after the `Arc` exists.
use std::sync::{Arc, Mutex, OnceLock, Weak};
trait WatcherImpl: Send + Sync {
    fn name(&self) -> String;
}
#[derive(Default)]
struct WatcherBase {
    mu: Mutex<Vec<u32>>,
    self_: OnceLock<Weak<dyn WatcherImpl>>,
}
impl WatcherBase {
    fn init(&self, self_: Weak<dyn WatcherImpl>) {
        let _ = self.self_.set(self_);
        self.mu.lock().unwrap().push(1);
    }
    fn self_impl(&self) -> Arc<dyn WatcherImpl> {
        match self.self_.get() {
            Some(w) => w.upgrade().expect("alive"),
            None => panic!("init was not called"),
        }
    }
}
struct Inotify {
    base: WatcherBase,
    fd: i32,
}
impl WatcherImpl for Inotify {
    fn name(&self) -> String {
        format!("inotify fd {}", self.fd)
    }
}
fn new_backend() -> Arc<Inotify> {
    let b = Arc::new(Inotify { base: WatcherBase::default(), fd: 3 });
    let self_: Weak<Inotify> = Arc::downgrade(&b);
    let self_impl: Weak<dyn WatcherImpl> = self_.clone();
    b.base.init(self_impl);
    b
}
fn main() {
    let b = new_backend();
    println!("{} {}", b.base.self_impl().name(), b.base.mu.lock().unwrap().len());
    println!("{} {}", Arc::strong_count(&b), Arc::weak_count(&b));
    let w = b.base.self_.get().unwrap().clone();
    drop(b);
    println!("{}", w.upgrade().is_none());
}
