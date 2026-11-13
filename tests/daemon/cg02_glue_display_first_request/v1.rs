// Regression: on a fresh daemon (per_unit 16), the unit with the Display
// glue must declare the fmt::Result C type itself. Exactly 15 pad fns
// put the glue in that position.
fn pad0(x: i64) -> i64 {
    x + 0
}
fn pad1(x: i64) -> i64 {
    x + 1
}
fn pad2(x: i64) -> i64 {
    x + 2
}
fn pad3(x: i64) -> i64 {
    x + 3
}
fn pad4(x: i64) -> i64 {
    x + 4
}
fn pad5(x: i64) -> i64 {
    x + 5
}
fn pad6(x: i64) -> i64 {
    x + 6
}
fn pad7(x: i64) -> i64 {
    x + 7
}
fn pad8(x: i64) -> i64 {
    x + 8
}
fn pad9(x: i64) -> i64 {
    x + 9
}
fn pad10(x: i64) -> i64 {
    x + 10
}
fn pad11(x: i64) -> i64 {
    x + 11
}
fn pad12(x: i64) -> i64 {
    x + 12
}
fn pad13(x: i64) -> i64 {
    x + 13
}
fn pad14(x: i64) -> i64 {
    x + 14
}
struct Q {
    x: i64,
    y: i64,
}
impl std::fmt::Display for Q {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<{}|{}>", self.x, self.y)
    }
}
fn main() {
    let p = Q { x: 1, y: 2 };
    println!("{}", p);
}
