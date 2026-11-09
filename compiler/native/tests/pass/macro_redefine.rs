macro_rules! m {
    () => {
        1
    };
}
fn first() -> i64 {
    m!()
}
macro_rules! m {
    () => {
        2
    };
}
fn main() {
    println!("{} {}", first(), m!());
}
