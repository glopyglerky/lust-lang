macro_rules! m {
    () => {
        1
    };
}
mod a;
macro_rules! m {
    () => {
        2
    };
}
mod b;
fn main() {
    println!("{} {}", a::v(), b::v());
}
