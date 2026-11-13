#[non_exhaustive]
pub struct Opts { pub level: i64 }
pub fn opts() -> Opts { Opts { level: 9 } }
#[non_exhaustive]
pub struct Pair(pub i64);
#[non_exhaustive]
pub struct Unit;
#[non_exhaustive]
pub enum Kind { A, B(i64), C { y: i64 } }
pub enum Shape { Dot, #[non_exhaustive] Rect { w: i64, h: i64 }, #[non_exhaustive] Wrap(i64) }
pub fn shape() -> Shape { Shape::Rect { w: 3, h: 4 } }
