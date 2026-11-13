macro_rules! seven {
    () => {
        7
    };
}
macro_rules! mk {
    () => {
        mod genmod;
    };
}
mk! {}

fn main() {
    println!("{}", genmod::v());
}
