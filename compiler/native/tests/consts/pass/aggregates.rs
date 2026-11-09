// ARCH 2.13: struct, enum, tuple, and array construction from
// const parts, plus a discriminated enum const (one of the
// three A1 shapes the gate names).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Shape {
    None,
    At(Point),
    Pair(i32, i32),
}
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
enum Code {
    Ok = 0,
    Bad = 1000,
}
const ORIGIN: Point = Point { x: 0, y: 0 };
const SHAPE: Shape = Shape::At(Point { x: 3, y: 4 });
const CODE: Code = Code::Bad;
const TUP: (u8, bool, Point) = (7, true, ORIGIN);
static SHAPES: [Shape; 3] = [Shape::None, SHAPE, Shape::Pair(1, 2)];
static ZEROS: [u16; 4] = [0; 4];
fn main() {
    let at = match SHAPES[1] {
        Shape::At(p) => p.x + p.y,
        _ => -1,
    };
    println!("{} {} {} {} {} {}", at, CODE as u32, TUP.0, TUP.1, TUP.2.x, ZEROS.len());
}
