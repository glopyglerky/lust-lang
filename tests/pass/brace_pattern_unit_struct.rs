// Brace patterns and literals fit unit structs and unit variants too.
struct U;
enum E {
    A(u8),
    C,
}
fn main() {
    let u = U {};
    let U {} = u;
    let U { .. } = U;
    for e in [E::A(3), E::C] {
        match e {
            E::A(n) => println!("a {}", n),
            E::C {} => println!("c"),
        }
    }
}
