// A trait method may be called on a float whose width is not fixed
// yet; the trait selection fixes it. Only inherent float methods are
// ambiguous there (rustc E0689).
trait Scaled {
    fn scaled(self) -> Self;
}

impl Scaled for f32 {
    fn scaled(self) -> Self {
        self * 10.0
    }
}

fn main() {
    let x = 2.0;
    let y = x.scaled();
    let z = x.clone();
    println!("{} {} {}", y, z, x.to_string());
}
