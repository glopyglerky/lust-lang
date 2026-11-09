// expect: L0001
fn main() {
    let a = Some(1);
    if (let Some(x) = a) {
        println!("{}", x);
    }
}
