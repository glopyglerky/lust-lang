// expect: L0120
fn main() {
    for i in 0..3 {
        'a: {
            if i == 1 {
                continue;
            }
            println!("{}", i);
        }
    }
}
