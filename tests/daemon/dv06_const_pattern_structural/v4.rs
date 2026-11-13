#[derive(Debug, Clone, Copy)]
struct C {
    n: u32,
}
const ONE: C = C { n: 1 };
fn is_one<T: Copy>(c: C, _t: T) -> bool {
    match c {
        ONE => true,
        _ => false,
    }
}
fn main() {
    println!("{} {}", is_one(C { n: 1 }, 0u8), is_one(C { n: 2 }, 'x'));
}
