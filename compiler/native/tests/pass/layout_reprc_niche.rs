// Layout fixes from the wave A1 review: repr(C) data enums are a tag
// plus a union of variant structs, arrays carry their element niche,
// a u128 payload can sit beside an align-8 niche variant, negative
// discriminants keep the tag niche, and unit fields / 128-bit tags
// have exact C sizes.
use std::mem::{align_of, size_of};

#[repr(C)]
#[derive(Debug, PartialEq)]
enum C1 {
    A(u8, u64),
    B,
}

#[repr(C)]
enum C3 {
    A(i16, f64),
}

#[repr(C, u8)]
#[derive(Debug, PartialEq)]
enum C6 {
    A(bool, u8),
    B(Option<char>, u128, u128),
}

#[repr(u8)]
enum I1 {
    A(bool, u8),
    B(Option<char>, u128, u128),
}

#[allow(dead_code)]
enum G {
    A([bool; 4]),
    B(u8, u8, u8),
}

#[allow(dead_code)]
enum U1 {
    A(u128),
    B([u64; 2], (bool, u64)),
}

enum Neg {
    A = -1,
    B = 127,
}

enum Wrap {
    A = 0,
    B = 255,
}

#[repr(i128)]
#[derive(Debug, PartialEq)]
enum Wide {
    X(u8) = -5,
    Y = 100,
}

#[repr(i128)]
enum WideVals {
    Lo = -170141183460469231731687303715884105728,
    Hi = 170141183460469231731687303715884105727,
}

struct WithUnit {
    a: (),
    b: u64,
    c: u8,
}

fn main() {
    // Sizes that differ between the tag-in-variant and tag-plus-union forms.
    println!("{} {} {} {}", size_of::<C1>(), size_of::<C3>(), size_of::<C6>(), size_of::<I1>());
    println!("{} {}", align_of::<C6>(), align_of::<C3>());
    println!("{:?} {:?}", C1::A(3, 70000), C6::B(Some('x'), 5, 6) == C6::B(Some('x'), 5, 6));

    // Array element niches.
    println!(
        "{} {} {} {}",
        size_of::<Option<[bool; 2]>>(),
        size_of::<Option<[char; 3]>>(),
        size_of::<Option<[bool; 0]>>(),
        size_of::<G>()
    );
    let some: Option<[bool; 2]> = Some([true, false]);
    println!("{} {}", some.unwrap()[0], size_of::<Option<Option<[bool; 2]>>>());

    // Align-16 other variant in a niche-filled enum.
    println!("{} {}", size_of::<U1>(), align_of::<U1>());

    // Negative discriminants keep the tag niche.
    println!("{} {} {} {}", size_of::<Option<Neg>>(), size_of::<Option<Wrap>>(), Neg::A as i32, Wrap::B as u32);
    let n: Option<Neg> = Some(Neg::B);
    println!("{}", n.unwrap() as i32);

    // 128-bit tags.
    println!("{} {:?} {} {}", size_of::<Wide>(), Wide::X(7), Wide::X(7) == Wide::Y, WideVals::Hi as u128);
    println!("{} {}", WideVals::Lo as i128, size_of::<Option<WideVals>>());

    // A direct unit field takes no space.
    let s = WithUnit { a: (), b: 41, c: 3 };
    let t = ((), 7u8, 9u64);
    println!("{} {} {} {} {}", size_of::<WithUnit>(), s.b + 1, size_of::<((), u8, u64)>(), t.1, t.2);
    let mut v: Vec<WithUnit> = Vec::new();
    v.push(WithUnit { a: (), b: 1, c: 2 });
    v.push(s);
    println!("{} {}", v[0].b, v[1].b);
}
