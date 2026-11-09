trait Marker {}
enum E {
    A(u32),
    B,
}
impl E {
    fn val(&self) -> u32 {
        match self {
            E::A(n) => *n,
            E::B => 0,
        }
    }
    fn val_plus(&self) -> u32 {
        self.val() + 1
    }
}
fn main() {
    let a = E::A(9);
    let b = E::B;
    let r = &a;
    println!("{} {} {}", a.val_plus(), b.val(), r.val_plus());
}
