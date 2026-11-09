// panic: called `Result::unwrap()` on an `Err` value: AccessError
// `try_with` on a destroyed key is Err(AccessError): Debug prints its
// name (no padding), Display "already destroyed" (padded), it is Copy and
// Eq, and unwrap panics with it.
struct D(u8);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
        if self.0 == 2 {
            println!("{:?}", FIRST.try_with(|f| f.0));
            let e = FIRST.try_with(|f| f.0);
            let copy = e;
            println!("{:?} {:?} {}", e, copy, e == copy);
            let err = e.unwrap_err();
            println!("[{:>15?}] [{:#?}] [{}] [{:>20}]", err, err, err, err);
            let v = FIRST.try_with(|f| f.0).unwrap();
            println!("never {}", v);
        }
    }
}
thread_local! {
    static FIRST: D = D(1);
    static SECOND: D = D(2);
}
fn main() {
    SECOND.with(|s| println!("touch {}", s.0));
    FIRST.with(|f| println!("touch {}", f.0));
    println!("{:?}", FIRST.try_with(|f| f.0));
}
