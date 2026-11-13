// Self:: paths to associated items: pub items from an impl in another
// module, private items from the own module, and pub(super) items of a
// child-module impl from the parent.
mod a {
    pub struct S {
        x: i32,
    }
    impl S {
        pub const C: i32 = 3;
        const D: i32 = 4;
        pub fn new() -> S {
            S { x: 1 }
        }
        pub fn get(&self) -> i32 {
            self.x
        }
        fn hidden(&self) -> i32 {
            Self::D + self.x
        }
        pub fn shown(&self) -> i32 {
            Self::hidden(self) + Self::C + Self::helper()
        }
    }
    mod imp {
        impl super::S {
            pub(super) fn helper() -> i32 {
                10
            }
        }
    }
    pub fn via_child() -> i32 {
        S::helper()
    }
}
impl a::S {
    fn other(&self) -> i32 {
        Self::get(self) + Self::C + Self::new().get()
    }
    fn matches(n: i32) -> i32 {
        match n {
            Self::C => 1,
            _ => 0,
        }
    }
}
fn main() {
    let s = a::S::new();
    println!("{} {} {} {}", s.other(), s.shown(), a::S::matches(3), a::via_child());
}
