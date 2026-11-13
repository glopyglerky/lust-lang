// expect: L0610
// rustc E0282: `*x > 7` leaves `?T: PartialOrd<{integer}>` pending; an
// operator never binds its open left operand, and nothing else fixes `v`.
fn main() {
    let v = Vec::new();
    let b = v.iter().any(|x| *x > 7);
    println!("{b} {}", v.len());
}
