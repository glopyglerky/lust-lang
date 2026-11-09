// expect: L0810
extern "C" fn one() -> i64 {
    1
}

fn main() {
    println!("{}", one());
}
