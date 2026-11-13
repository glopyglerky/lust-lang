// expect: L0405
async fn one() -> i64 {
    1
}

fn main() {
    let _f = one();
    println!("hi");
}
