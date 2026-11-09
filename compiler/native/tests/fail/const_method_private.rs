// expect: L0510
// rustc E0624: a private const method called from a const outside its module.
mod m {
    pub struct W(pub u32);
    impl W {
        const fn get(&self) -> u32 {
            self.0
        }
    }
}
const X: u32 = m::W(3).get();
fn main() {
    println!("{}", X);
}
