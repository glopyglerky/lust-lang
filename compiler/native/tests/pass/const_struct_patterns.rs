// Structural-match const patterns: consts of types that derive
// PartialEq match field by field (rustc's const-to-pattern).
use std::fmt;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Category(u32);

#[allow(non_upper_case_globals)]
impl Category {
    pub const Warning: Category = Category(0);
    pub const Error: Category = Category(1);
    pub const Suggestion: Category = Category(2);
}

impl fmt::Debug for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Warning => f.write_str("Warning"),
            Self::Error => f.write_str("Error"),
            Self::Suggestion => f.write_str("Suggestion"),
            Self(raw) => write!(f, "Category({raw})"),
        }
    }
}

#[derive(Debug)]
struct Message {
    code: u32,
    category: Category,
}

#[derive(PartialEq, Eq, Debug)]
enum Shape {
    Unit,
    Pair(u8, char),
    Named { w: i32, label: &'static str },
}

#[derive(PartialEq, Debug)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(PartialEq, Debug)]
struct Wrap<T>(T);

const PAIR: Shape = Shape::Pair(3, 'z');
const NAMED: Shape = Shape::Named { w: -4, label: "box" };
const ORIGIN: Point = Point { x: 0.0, y: 0.0 };
const SOME_PAIR: Option<Shape> = Some(Shape::Pair(1, 'a'));
const OK7: Result<u8, &str> = Ok(7);
const ERR: Result<u8, &str> = Err("bad");
const TUP: (u8, bool, &str) = (1, true, "t");
const ARR: [u16; 3] = [1, 2, 3];
const REP: [u8; 4] = [9; 4];
const SL: &[u8] = &[1, 2];
const EMPTY: &[i32] = &[];
const BYTES: &[u8] = b"hi";
const REF_POINT: &Point = &Point { x: 1.0, y: 2.0 };
const WRAP: Wrap<Option<char>> = Wrap(Some('q'));
const NESTED: (Category, [Option<u8>; 2]) = (Category(5), [None, Some(1)]);

fn shape(s: &Shape) -> &'static str {
    match *s {
        PAIR => "pair 3 z",
        NAMED => "named",
        Shape::Pair(..) => "other pair",
        Shape::Named { .. } => "other named",
        Shape::Unit => "unit",
    }
}

fn opt(o: Option<Shape>) -> u8 {
    match o {
        SOME_PAIR => 1,
        Some(_) => 2,
        None => 3,
    }
}

fn res(r: Result<u8, &str>) -> u8 {
    match r {
        OK7 => 1,
        ERR => 2,
        Ok(_) | Err(_) => 3,
    }
}

fn slice(s: &[u8]) -> u8 {
    match s {
        SL => 1,
        BYTES => 2,
        [] => 3,
        _ => 4,
    }
}

fn main() {
    for c in [Category::Warning, Category::Error, Category::Suggestion, Category(9)] {
        println!("{c:?}");
    }
    println!("{:?}", Message { code: 2304, category: Category::Error });
    for s in [Shape::Unit, Shape::Pair(3, 'z'), Shape::Pair(3, 'y'), Shape::Named { w: -4, label: "box" }, Shape::Named { w: -4, label: "bo" }] {
        println!("{:?} -> {}", s, shape(&s));
    }
    // -0.0 equals 0.0, as `==` says.
    for p in [Point { x: 0.0, y: -0.0 }, Point { x: 0.0, y: 1.0 }] {
        println!("{}", match p { ORIGIN => "origin", _ => "elsewhere" });
    }
    println!("{} {} {}", opt(Some(Shape::Pair(1, 'a'))), opt(Some(Shape::Unit)), opt(None));
    println!("{} {} {} {}", res(Ok(7)), res(Ok(8)), res(Err("bad")), res(Err("ok")));
    for t in [(1, true, "t"), (1, true, "u"), (2, true, "t")] {
        println!("{}", match t { TUP => "tup", (_, b, _) => if b { "true" } else { "false" } });
    }
    for a in [[1, 2, 3], [1, 2, 4]] {
        println!("{}", match a { ARR => 1, [_, _, _] => 2 });
    }
    println!("{}", match [9u8; 4] { REP => "rep", _ => "no" });
    println!("{} {} {} {}", slice(&[1, 2]), slice(b"hi"), slice(&[]), slice(&[1]));
    let e: &[i32] = &[];
    println!("{}", match e { EMPTY => "empty", _ => "full" });
    let r = &Point { x: 1.0, y: 2.0 };
    println!("{}", match r { REF_POINT => "ref", _ => "no" });
    println!("{}", match Wrap(Some('q')) { WRAP => 1, Wrap(_) => 2 });
    println!("{}", match (Category(5), [None, Some(1)]) { NESTED => "nested", _ => "no" });
    // Next to bindings, in or-patterns, inside other patterns, with guards.
    for v in [Some(Category::Error), Some(Category(7)), None] {
        let s = match v {
            Some(Category::Warning | Category::Error) => "bad".to_string(),
            Some(c @ Category(n)) if n > 5 => format!("{c:?} big"),
            Some(c) => format!("{c:?}"),
            None => "none".to_string(),
        };
        println!("{s}");
    }
    if let Some(Category::Error) = Some(Category(1)) {
        println!("if let");
    }
    let items = [Category(0), Category(1), Category(1)];
    println!("{}", items.iter().filter(|c| matches!(c, &&Category::Error)).count());
}
