// The iterator expansion's own values (a vec::IntoIter, a collected Vec)
// drop when a closure inside it panics.
struct D(i32);
impl Drop for D { fn drop(&mut self) { println!("drop {}", self.0); } }
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let r = std::panic::catch_unwind(|| {
        let v = vec![D(1), D(2), D(3), D(4)];
        let s: i32 = v.into_iter().map(|d| { if d.0 == 2 { panic!("two") } d.0 }).sum();
        s
    });
    println!("{}", r.is_err());
    let r = std::panic::catch_unwind(|| {
        let v = vec![D(5), D(6), D(7)];
        let w: Vec<D> = v.into_iter().map(|d| { if d.0 == 7 { panic!("seven") } D(d.0 * 10) }).collect();
        w.len()
    });
    println!("{}", r.is_err());
    let r = std::panic::catch_unwind(|| { let v = vec![1u8, 200, 100]; v.iter().map(|x| *x).fold(0u8, |a, b| a + b) });
    println!("{}", r.is_err());
}
