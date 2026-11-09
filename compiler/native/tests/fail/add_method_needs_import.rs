// expect: L0104
// rustc E0599: the operator traits are not in the prelude; `+` works, `.add()` needs the import.
struct P(i32);
impl std::ops::Add for P {
    type Output = P;
    fn add(self, o: P) -> P {
        P(self.0 + o.0)
    }
}
fn main() {
    let a = P(1) + P(2);
    let b = a.add(P(3));
    println!("{}", b.0);
}
