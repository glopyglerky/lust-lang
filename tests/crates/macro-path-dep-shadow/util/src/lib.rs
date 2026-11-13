#[macro_export]
macro_rules! helper {
    () => {
        40
    };
}
#[macro_export]
macro_rules! m {
    () => {
        $crate::helper!() + $crate::f()
    };
}
pub fn f() -> i32 {
    2
}
