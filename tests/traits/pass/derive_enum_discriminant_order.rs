// ARCH 2.2 derives: derived PartialOrd/Ord order enum variants by
// discriminant value (signed, per repr), then by fields.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
#[repr(i8)]
enum G { A = 5, B = -3, C, D(u8) = 100 }
fn main() {
    let w = [G::A, G::B, G::C, G::D(1), G::D(0)];
    for x in &w {
        for y in &w {
            print!("{:?} ", x.cmp(y));
        }
        println!();
    }
}
