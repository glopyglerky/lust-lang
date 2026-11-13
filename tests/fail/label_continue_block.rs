// expect: L0120
fn main() {
    let x = 'a: {
        continue 'a;
    };
    println!("{}", x);
}
