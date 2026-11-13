#[path = "sub/m.rs"]
mod m;
fn main() {
    println!("{}", module_path!());
    m::f();
    m::inner::g();
}
