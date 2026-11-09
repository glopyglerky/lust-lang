// expect: L0403
#[derive(Default)]
enum E {
    #[default]
    #[default]
    A,
    B,
}

fn main() {
    let _e = E::default();
    println!("x");
}
