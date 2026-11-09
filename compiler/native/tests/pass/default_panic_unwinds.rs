// A single-file build uses rustc's default panic strategy: panics unwind, as with plain
// `rustc file.rs`. Drop runs during unwinding, catch_unwind and a joined thread recover,
// cfg!(panic) says "unwind", and the last panic exits 101. Under the old panic=abort
// default the first panic aborted (exit 134) and printed none of this. difftest's
// PANIC=abort mode runs this case with -C panic=abort on both sides.
struct G(&'static str);
impl Drop for G {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn deep(v: &[u8], i: usize) -> u8 {
    let _g = G("deep");
    v[i]
}

fn main() {
    let _a = G("main");
    println!("unwind {}", cfg!(panic = "unwind"));
    let r = std::panic::catch_unwind(|| {
        let _b = G("closure");
        deep(&[1, 2], 5)
    });
    println!("caught {}", r.is_err());
    let j = std::thread::spawn(|| {
        let _c = G("thread");
        let n: Option<u8> = None;
        n.unwrap()
    })
    .join();
    println!("joined {}", j.is_err());
    panic!("last panic");
}
