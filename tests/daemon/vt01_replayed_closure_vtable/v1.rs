use std::sync::{Arc, Mutex};

type ThreadMain<'a, T> = Box<dyn FnOnce() -> T + Send + 'a>;

// A generic definer's `impl FnOnce` return (a closure of its instance),
// boxed into a `dyn` inside a closure of another generic's instance: the
// vtable names a closure that exists only once the definer's instance is
// made, a round after the body that coerces it.
fn take_once<F: FnOnce() -> T, T>(slot: &Arc<Mutex<Option<F>>>) -> impl FnOnce() -> T + use<F, T> {
    let slot = Arc::clone(slot);
    move || {
        let f = slot.lock().unwrap().take();
        f.expect("taken once")()
    }
}

fn retry<H>(mut f: impl FnMut() -> Result<H, String>) -> Result<H, String> {
    match f() {
        Ok(h) => Ok(h),
        Err(_) => f(),
    }
}

struct Runner {
    name: String,
}

impl Runner {
    fn spawn<F, T>(&self, f: F) -> T
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        self.start(f, |name, run| {
            let _ = name;
            Ok(run())
        })
    }

    fn start<'a, F, T, H>(&self, f: F, mut clone: impl FnMut(String, ThreadMain<'a, T>) -> Result<H, String>) -> H
    where
        F: FnOnce() -> T + Send + 'a,
        T: 'a,
    {
        let slot = Arc::new(Mutex::new(Some(f)));
        match retry(|| clone(self.name.clone(), Box::new(take_once(&slot)))) {
            Ok(h) => h,
            Err(e) => panic!("{e}"),
        }
    }
}

fn main() {
    let r = Runner { name: "t".to_string() };
    println!("{}", r.spawn(|| 1 + 2));
    println!("{}", r.spawn(|| "x".to_string()));
}
