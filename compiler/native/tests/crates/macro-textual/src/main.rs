macro_rules! v {
    () => {
        1
    };
}
mod a;
macro_rules! v {
    () => {
        2
    };
}
#[macro_use]
mod defs;
mod user;

fn main() {
    println!("{} {} {} {}", a::f(), a::b::g(), twice!(4), user::f());
}
