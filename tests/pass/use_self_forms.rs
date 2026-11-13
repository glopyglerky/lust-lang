// `self` in a group binds the module; keyword paths bind with a rename.
mod a {
    pub fn b() -> i32 {
        7
    }
}
mod m {
    use crate::a::{self};
    use super as up;
    use crate as top;
    pub fn g() -> i32 {
        a::b() + up::a::b() + top::a::b()
    }
}

fn main() {
    println!("{}", m::g());
}
