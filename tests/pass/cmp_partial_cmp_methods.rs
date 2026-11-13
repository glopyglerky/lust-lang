// Comparison trait methods in method form: `partial_cmp`, `cmp`,
// `eq`/`ne`, and the `lt`/`le`/`gt`/`ge` defaults, in templates
// (the lang stub signatures) and on concrete scalars.
use std::cmp::Ordering;

fn less<T: PartialOrd>(a: &T, b: &T) -> bool {
    a.partial_cmp(b) == Some(Ordering::Less)
}

fn total<T: Ord>(a: &T, b: &T) -> bool {
    a.cmp(b) == Ordering::Greater
}

fn unordered<T: PartialOrd>(a: &T, b: &T) -> bool {
    a.partial_cmp(b).is_none()
}

fn rel<T: PartialOrd>(a: &T, b: &T) -> bool {
    a.lt(b) && b.gt(a) && a.le(b) && b.ge(a)
}

fn same<T: PartialEq>(a: &T, b: &T) -> bool {
    a.eq(b) || a.ne(b)
}

fn via_super<T: PartialOrd<U>, U>(a: &T, b: &U) -> bool {
    a.eq(b)
}

fn main() {
    println!("{}", less(&1u8, &2u8));
    println!("{}", total(&3u8, &2u8));
    println!("{} {}", unordered(&f64::NAN, &1.0), unordered(&1.0f64, &2.0));
    println!("{}", rel(&1u8, &2u8));
    println!("{} {}", same(&1u8, &1u8), 2u8.eq(&2u8));
    println!("{}", via_super(&1u8, &1u8));
    println!("{:?} {:?}", 1u8.partial_cmp(&2u8), 3i32.cmp(&2i32));
    println!("{} {}", f64::NAN.eq(&f64::NAN), f64::NAN.ne(&f64::NAN));
    println!("{:?} {:?}", true.cmp(&false), 'a'.partial_cmp(&'b'));
}
