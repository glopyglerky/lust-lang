// A root type alias named like a primitive shadows it in the root module
// (let types, casts, turbofish, literals' inferred type, associated
// consts), as in rustc; in a module that does not import it, the name is
// still the primitive.
#[allow(non_camel_case_types)]
type u8 = u16;
mod m {
    pub fn f() -> u32 {
        let x: u8 = 255;
        x.wrapping_add(1) as u32
    }
}
fn main() {
    let a = 255 as u8;
    let b: u8 = 300;
    let x: u8 = 255;
    println!("{} {} {} {}", a.wrapping_add(1), m::f(), std::mem::size_of::<u8>(), b);
    println!("{} {}", x.wrapping_add(1), u8::MAX);
}
