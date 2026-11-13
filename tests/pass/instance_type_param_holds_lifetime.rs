// A generic fn whose type param is bound, in an instance, to a type that holds a lifetime
// (`&'static str`, `Box<dyn Any + Send>`, a scoped borrow). `lust build` once borrow-checked
// each instance with coarser call rules than the template and rejected these (L0203 "cannot
// prove the callee's T: 'a bound", L0306/L0307 on `self`) where `lust check` accepted them.
// Shape of ts-rust's goport_util GoThread::spawn / spawn_scoped (gopanic.rs).
use std::any::Any;
use std::thread::{Builder, JoinHandle, Scope, ScopedJoinHandle};

type ThreadMain<'a, T> = Box<dyn FnOnce() -> T + Send + 'a>;

#[derive(Default)]
struct GoThread {
    name: Option<String>,
}

impl GoThread {
    fn spawn<F, T>(self, f: F) -> JoinHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        self.start(f, Builder::spawn, std::thread::sleep)
    }

    fn spawn_scoped<'scope, 'env, F, T>(self, scope: &'scope Scope<'scope, 'env>, f: F) -> ScopedJoinHandle<'scope, T>
    where
        F: FnOnce() -> T + Send + 'scope,
        T: Send + 'scope,
    {
        self.start(f, |builder, run| builder.spawn_scoped(scope, run), std::thread::sleep)
    }

    fn start<'a, F, T, H>(
        &self,
        f: F,
        mut clone: impl FnMut(Builder, ThreadMain<'a, T>) -> std::io::Result<H>,
        mut sleep: impl FnMut(std::time::Duration),
    ) -> H
    where
        F: FnOnce() -> T + Send + 'a,
        T: 'a,
    {
        let mut f = Some(f);
        loop {
            let builder = match &self.name {
                Some(n) => Builder::new().name(n.clone()),
                None => Builder::new(),
            };
            match clone(builder, Box::new(f.take().unwrap())) {
                Ok(h) => return h,
                Err(e) => {
                    sleep(std::time::Duration::from_millis(1));
                    panic!("start: {e}");
                }
            }
        }
    }
}

// Free fn form, `Builder::spawn` as a value.
fn start_free<'a, F, T, H>(f: F, clone: impl FnOnce(Builder, ThreadMain<'a, T>) -> std::io::Result<H>) -> H
where
    F: FnOnce() -> T + Send + 'a,
    T: 'a,
{
    clone(Builder::new(), Box::new(f)).unwrap()
}

fn spawn_free<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    start_free(f, Builder::spawn)
}

// No threads: an `&self` method whose result is a type param, called from an instance
// where that param holds a lifetime.
struct G;

impl G {
    fn start<'a, F, T, H>(&self, f: F, mut clone: impl FnMut(Box<dyn FnOnce() -> T + 'a>) -> H) -> H
    where
        F: FnOnce() -> T + 'a,
        T: 'a,
    {
        clone(Box::new(f))
    }

    fn run<F: FnOnce() -> T + 'static, T: 'static>(self, f: F) -> T {
        self.start(f, |m| m())
    }
}

fn apply<A, B>(a: A, f: fn(A) -> B) -> B {
    f(a)
}

fn main() {
    println!("{}", spawn_free(|| "free").join().unwrap());
    let h = GoThread { name: Some("named".to_string()) }.spawn(|| std::thread::current().name().map(str::len));
    println!("{:?}", h.join().unwrap());
    println!("{}", GoThread::default().spawn(|| "static str").join().unwrap());
    let any = GoThread::default().spawn(|| -> Box<dyn Any + Send> { Box::new(5u8) }).join().unwrap();
    println!("{:?}", any.downcast_ref::<u8>());

    let words = vec!["a".to_string(), "bc".to_string()];
    let lens = std::thread::scope(|s| {
        let w = &words[1];
        let len = GoThread::default().spawn_scoped(s, move || w.len());
        let word = GoThread::default().spawn_scoped(s, move || w.as_str());
        let lit = GoThread::default().spawn_scoped(s, || "scoped str");
        (len.join().unwrap(), word.join().unwrap(), lit.join().unwrap())
    });
    println!("{lens:?}");

    println!("{}", G.run(|| "no thread"));
    println!("{:?}", G.run(|| Some(&7)));

    // std fns with bounds as fn values.
    // `Box::new` and `Rc::new` as fn pointers are not supported yet (L0110).
    let a = apply(String::from("arc"), std::sync::Arc::new);
    println!("{a}");
    let o = Some("opt").map(Box::new).map(std::rc::Rc::new);
    println!("{o:?}");
    let t = Some(|| "via thread::spawn").map(std::thread::spawn).unwrap();
    println!("{}", t.join().unwrap());
}
