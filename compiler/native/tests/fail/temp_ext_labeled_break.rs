// expect: L0306
// rustc E0716: a labeled block `break` value is not extending.
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn mk(n: &'static str) -> D {
    D(n)
}
fn main() {
    let c = mk("c").0.len() == 1;
    let x = 'a: {
        if c {
            break 'a &mk("t1");
        }
        &mk("t2")
    };
    println!("{}", x.0);
}
