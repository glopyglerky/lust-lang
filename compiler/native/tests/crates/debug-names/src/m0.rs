#[derive(Debug)]
pub struct Stats {
    pub n: i64,
}

#[derive(Debug)]
pub struct Pair(pub i64, pub bool);

#[derive(Debug)]
pub struct Marker;

#[derive(Debug)]
pub struct Empty {}

#[derive(Debug)]
pub enum Shape {
    Dot,
    Circle(i64),
    Rect { w: i64, h: i64 },
}

#[derive(Debug)]
pub struct Wrap<T> {
    pub inner: T,
}

// A struct variant cannot be named from main today (its fields read
// as private in lust), so the value is built here.
pub fn rect() -> Shape {
    Shape::Rect { w: 2, h: 4 }
}
