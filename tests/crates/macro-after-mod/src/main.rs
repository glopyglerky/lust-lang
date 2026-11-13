mod child;
macro_rules! hi {
    () => {
        3
    };
}

fn main() {
    println!("{}", child::f());
}
