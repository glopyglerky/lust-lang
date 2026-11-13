macro_rules! m {
    () => {
        1
    };
}
mod b;
macro_rules! m {
    () => {
        2
    };
}
mod a;
fn main() {
    println!("{} {}", a::v(), b::v());
}
