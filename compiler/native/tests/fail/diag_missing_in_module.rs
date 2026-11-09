// expect: L0102
mod ata {
    pub struct Present;
}
struct Session {
    x: ata::Missing,
}
fn main() {}
