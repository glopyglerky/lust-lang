// Restricted visibility forms that name ancestors (or the item's own
// module) are valid on items, fields, methods, and imports.
mod pa {
    pub mod inner {
        pub(in crate::pa) fn f() -> i32 {
            1
        }
        pub(in crate::pa::inner) fn own() -> i32 {
            2
        }
        pub(self) fn me() -> i32 {
            3
        }
        pub(super) fn up() -> i32 {
            own() + me()
        }
        pub(crate) struct R {
            pub(in crate::pa) x: i32,
            pub(super) y: i32,
        }
        pub(in crate) fn mk() -> R {
            R { x: 4, y: 5 }
        }
        pub struct T;
        impl T {
            pub(in crate::pa) fn t(&self) -> i32 {
                6
            }
        }
        pub(in super) use self::f as f2;
    }
    pub fn read() -> i32 {
        let r = inner::mk();
        inner::f() + inner::up() + r.x + r.y + inner::T.t() + inner::f2()
    }
}
pub(crate) fn top() -> i32 {
    pa::read()
}
fn main() {
    println!("{}", top());
}
