#[path = "sub/m.rs"]
mod m;
fn main() {
    m::inner::g();
}
