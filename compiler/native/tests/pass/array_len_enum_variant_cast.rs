// Array lengths from a unit enum's discriminant (ts-rust frontend/parser
// ParsingContext::ALL: `[ParsingContext; ParsingContext::Count as usize]`).
#[derive(Clone, Copy, Debug)]
enum C { A, B, Count }
impl C {
    const ALL: [C; C::Count as usize] = [C::A, C::B];
}
#[repr(u8)]
#[derive(Debug)]
enum R { X = 3, Y, Z = 10 }
#[allow(dead_code)]
#[derive(Debug)]
enum N { M = -2, O, P }
mod m {
    #[allow(dead_code)]
    pub enum C { Q = 7, Count = 5 }
}
const T: [u8; R::Y as usize] = [1; 4];
fn main() {
    let a: [u8; R::Z as usize - R::X as usize] = [0; 7];
    let b = [0u16; m::C::Count as usize];
    let c = [1i8; (N::P as i8 + 3) as usize];
    println!("{:?} {} {} {} {}", C::ALL, T.len(), a.len(), b.len(), c.len());
}
