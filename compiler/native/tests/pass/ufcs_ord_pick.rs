// `Ord::max`, `min`, and `clamp` named through the trait, at builtin
// and structural types, in generic and concrete code.
fn hi<T: Ord>(a: T, b: T) -> T { <T as Ord>::max(a, b) }
fn lo<T: Ord>(a: T, b: T) -> T { Ord::min(a, b) }
fn mid<T: Ord>(a: T, l: T, h: T) -> T { Ord::clamp(a, l, h) }
fn main() {
    println!("{} {} {}", hi(-2i64, 7), lo('a', 'b'), hi(true, false));
    println!("{:?} {:?}", hi((1u8, 2u8), (1, 3)), lo(Some(4u16), None));
    println!("{:?} {:?}", hi([1u8, 9], [2, 0]), mid((5i8, 1i8), (0, 0), (3, 3)));
    println!("{} {}", <u32 as Ord>::max(3, 9), Ord::min(-1i128, 4));
    println!("{:?}", <(u8, char) as Ord>::min((2, 'x'), (2, 'a')));
}
