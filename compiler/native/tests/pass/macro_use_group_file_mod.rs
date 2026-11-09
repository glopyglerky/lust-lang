// `pub(crate) use {a, b};` of macro_rules names in a file module: both
// macros reach other modules by path and by `use`.
#[path = "macro_use_group_file_mod.rs.inc"]
mod macros;

mod user {
    use crate::macros::first;

    pub fn f() -> i32 {
        first!(Ok::<i32, ()>(40)) + crate::macros::second!()
    }
}

fn main() {
    println!("{}", user::f());
}
