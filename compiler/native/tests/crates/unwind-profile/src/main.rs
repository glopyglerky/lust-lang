// A Cargo package with no `panic` setting: the dev profile unwinds, so lust
// builds it with panic=unwind (W-UNWIND) and the caught panic recovers.
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let r = std::panic::catch_unwind(|| {
        let _d = D("inner");
        panic!("caught");
    });
    println!("recovered {}", r.is_err());
    let t = std::thread::spawn(|| panic!("in thread")).join();
    println!("thread err {}", t.is_err());
}
