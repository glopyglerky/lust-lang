// expect: L0308
fn main() {
    let x = 0;
    let mut c = move || {
        x += 1;
        x
    };
    println!("{}", c());
}
