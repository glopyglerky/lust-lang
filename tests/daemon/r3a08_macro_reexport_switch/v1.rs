#[macro_export]
macro_rules! make_one {
    () => {
        pub fn made() -> u32 {
            1
        }
    };
}
#[macro_export]
macro_rules! make_two {
    () => {
        pub fn made() -> u32 {
            2
        }
    };
}
mod exports {
    pub use crate::make_one as make;
}
mod user {
    crate::exports::make!();
}
fn main() {
    println!("{}", user::made());
}
