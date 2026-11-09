// std::process::exit runs the calling thread's thread-local destructors
// (glibc's exit() does, for Rust std on Linux), but not main's locals.
struct D(&'static str);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

thread_local! {
    static KEY: D = D("key");
}

fn main() {
    let _local = D("local");
    KEY.with(|k| println!("touch {}", k.0));
    println!("exit");
    std::process::exit(3);
}
