#[macro_export]
macro_rules! exported_answer {
    () => {
        42
    };
}

pub fn answer() -> i32 {
    42
}
