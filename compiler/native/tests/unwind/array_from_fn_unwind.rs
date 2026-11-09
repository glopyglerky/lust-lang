// A panic inside `std::array::from_fn`'s callback drops the
// elements already built front to back, then the closure, as
// core's buffer guard does.
struct Noisy(u8);
impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn main() {
    let r = std::panic::catch_unwind(|| {
        let a: [Noisy; 4] = std::array::from_fn(|i| {
            if i == 2 {
                panic!("boom");
            }
            Noisy(i as u8)
        });
        println!("{}", a.len());
    });
    println!("caught {}", r.is_err());
}
