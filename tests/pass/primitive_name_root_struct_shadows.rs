// A root struct named like a primitive shadows it in the root module, with
// its own inherent methods; a module that does not import it still sees
// the primitive (rustc).
#[allow(non_camel_case_types, dead_code)]
#[derive(Debug, Default, Clone, Copy)]
struct u8 {
    v: i32,
}
impl u8 {
    fn get(self) -> i32 {
        self.v * 2
    }
}
mod m {
    pub fn f() -> u8 {
        200
    }
}
fn main() {
    let x: u8 = u8 { v: 3 };
    println!("{:?} {} {}", x, x.get(), m::f());
}
