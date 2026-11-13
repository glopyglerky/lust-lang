// Captures drop in capture order (first use in the body), when the
// closure drops. An FnOnce call drops them right after the body.
struct Noisy(&'static str);
impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn run<F: FnOnce()>(f: F) {
    f();
    println!("after call");
}
fn main() {
    let a = Noisy("a");
    let b = Noisy("b");
    run(move || {
        let _ = &b;
        let _ = &a;
        println!("in body");
    });
    let c = Noisy("c");
    let d = Noisy("d");
    {
        let keep = move || {
            let _ = &c;
            let _ = &d;
        };
        keep();
        println!("scope end");
    }
    let e = Noisy("e");
    let eat = move || {
        let inner = e;
        println!("ate {}", inner.0);
    };
    eat();
    println!("end");
}
