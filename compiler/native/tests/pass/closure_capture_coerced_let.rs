// A captured value used as a `let` initializer at a coercion site inside
// a closure (ts-rust contentmapper/hostimpl.rs tests `FakeSpawner::spawn`:
// `let server: Arc<dyn ReadWriteCloser> = server;`). The env field keeps
// the captured type; lust gave it the pattern's type and tcc failed.
use std::sync::Arc;

trait Rw: Send + Sync {
    fn name(&self) -> String;
}

struct Pipe {
    n: u32,
}

impl Rw for Pipe {
    fn name(&self) -> String {
        format!("pipe{}", self.n)
    }
}

trait Show {
    fn show(&self) -> String;
}

impl Show for u32 {
    fn show(&self) -> String {
        format!("u{self}")
    }
}

struct S {
    b: Box<u32>,
    n: String,
}

fn seven() -> i32 {
    7
}

fn main() {
    let (client, server) = (Arc::new(Pipe { n: 1 }), Arc::new(Pipe { n: 2 }));
    let t = std::thread::spawn(move || {
        let server: Arc<dyn Rw> = server;
        server.name()
    });
    println!("{}", t.join().unwrap());
    let c = move || {
        let other: Arc<dyn Rw> = client;
        other.name()
    };
    println!("{}", c());
    let f = seven;
    let g = move || {
        let p: fn() -> i32 = f;
        p() + 1
    };
    println!("{}", g());
    let s = S { b: Box::new(4), n: String::from("nm") };
    let d = move || {
        let b: Box<dyn Show> = s.b;
        b.show() + &s.n
    };
    println!("{}", d());
}
