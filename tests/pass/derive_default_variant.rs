#[derive(Default)]
enum E {
    #[default]
    A,
    B,
}

fn main() {
    let _e = E::default();
    println!("hi");
}
