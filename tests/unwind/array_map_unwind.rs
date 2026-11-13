// A panic inside `[T; N]::map`'s closure (R22): the outputs built so far
// drop front to back, then the inputs not yet taken, then the closure,
// as core's guard and draining iterator do.
struct In(u8);
impl Drop for In {
    fn drop(&mut self) {
        println!("drop in {}", self.0);
    }
}
struct Out(u8);
impl Drop for Out {
    fn drop(&mut self) {
        println!("drop out {}", self.0);
    }
}
fn main() {
    let r = std::panic::catch_unwind(|| {
        let tag = Out(99);
        let a = [In(0), In(1), In(2), In(3)];
        let b = a.map(move |x| {
            let _t = &tag;
            if x.0 == 2 {
                panic!("boom");
            }
            Out(x.0 + 10)
        });
        println!("{}", b.len());
    });
    println!("caught {}", r.is_err());
}
