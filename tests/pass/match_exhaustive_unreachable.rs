// An exhaustive match's otherwise edge is `__builtin_unreachable()` in release C without debug
// assertions or overflow checks (rustc's `unreachable`): every match shape here must still pick
// the right arm. Enums, nested and same-field arms, Option and Result, full integer and char
// ranges, tuples, or-patterns, guards, slices, references and an uninhabited error.
use std::convert::Infallible;

#[derive(Clone, Copy, Debug)]
enum Fixed {
    Owned(u32),
    Static(u32),
}

enum Shape {
    Unit,
    Pair(i64, i64),
    Named { w: u16, h: u16 },
    Nested(Option<Box<Shape>>),
}

fn value(f: Fixed) -> u32 {
    match f {
        Fixed::Owned(v) => v,
        Fixed::Static(v) => v,
    }
}

fn area(s: &Shape) -> i64 {
    match s {
        Shape::Unit => 0,
        Shape::Pair(a, b) => a * b,
        Shape::Named { w, h } => i64::from(*w) * i64::from(*h),
        Shape::Nested(None) => -1,
        Shape::Nested(Some(inner)) => 1 + area(inner),
    }
}

fn byte_class(b: u8) -> &'static str {
    match b {
        0..=31 | 127 => "control",
        b'0'..=b'9' => "digit",
        b'a'..=b'z' | b'A'..=b'Z' => "letter",
        32..=126 => "other",
        128..=255 => "high",
    }
}

fn char_class(c: char) -> u8 {
    match c {
        '\0'..='\u{7f}' => 1,
        '\u{80}'..='\u{7ff}' => 2,
        '\u{800}'..='\u{ffff}' => 3,
        '\u{10000}'..=char::MAX => 4,
    }
}

fn sign_pair(p: (bool, i8)) -> i32 {
    match p {
        (true, i8::MIN..=-1) => -2,
        (true, 0) => 0,
        (true, 1..=i8::MAX) => 2,
        (false, x) if x % 2 == 0 => 10,
        (false, _) => 11,
    }
}

fn slice_kind(s: &[u16]) -> usize {
    match s {
        [] => 0,
        [x] => usize::from(*x),
        [a, b] => usize::from(*a) + usize::from(*b),
        [first, .., last] => usize::from(*first) * 1000 + usize::from(*last),
    }
}

fn unwrap_ok(r: Result<u64, Infallible>) -> u64 {
    match r {
        Ok(v) => v,
        Err(e) => match e {},
    }
}

fn opt_res(o: Option<Result<i32, String>>) -> i32 {
    match o {
        None => 0,
        Some(Ok(v)) => v,
        Some(Err(s)) => -(s.len() as i32),
    }
}

fn main() {
    let n = std::env::args().count() as u32; // 3 under the runner
    let fs = [Fixed::Owned(n), Fixed::Static(n * 7), Fixed::Owned(9)];
    println!("{:?}", fs.iter().map(|f| value(*f)).collect::<Vec<_>>());
    let deep = Shape::Nested(Some(Box::new(Shape::Nested(Some(Box::new(Shape::Pair(6, i64::from(n))))))));
    let shapes = [Shape::Unit, Shape::Pair(-4, 5), Shape::Named { w: 300, h: 7 }, Shape::Nested(None), deep];
    println!("{:?}", shapes.iter().map(area).collect::<Vec<_>>());
    let mut counts = std::collections::BTreeMap::new();
    for b in 0..=255u8 {
        *counts.entry(byte_class(b)).or_insert(0) += 1;
    }
    println!("{counts:?}");
    println!("{:?}", ['a', 'é', '€', '😀', '\u{10ffff}'].map(char_class));
    let pairs: Vec<i32> = [-128i8, -1, 0, 1, 127].iter().flat_map(|&x| [sign_pair((true, x)), sign_pair((false, x))]).collect();
    println!("{pairs:?}");
    let v: Vec<u16> = (1..=n as u16 + 3).collect();
    println!("{} {} {} {}", slice_kind(&[]), slice_kind(&v[..1]), slice_kind(&v[..2]), slice_kind(&v));
    println!("{}", unwrap_ok(Ok(u64::from(n) << 40)));
    println!("{} {} {}", opt_res(None), opt_res(Some(Ok(-7))), opt_res(Some(Err("four".to_string()))));
}
