use std::panic;

struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn main() {
    println!("panic cfg: {}", if cfg!(panic = "unwind") { "unwind" } else { "abort" });
    println!("debug_assertions: {}", cfg!(debug_assertions));
    let r = panic::catch_unwind(|| {
        let _d = D("inner");
        if std::env::args().count() < 100 {
            panic!("boom {}", 7);
        }
        5
    });
    println!("caught: {}", r.is_err());
    let v = panic::catch_unwind(|| 9).unwrap();
    println!("ok: {v}");
    debug_assert!(std::env::args().count() > 100, "debug assert ran");
    println!("end");
}
