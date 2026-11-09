// expect: L0113
fn main() {
    let v = vec![1, 2];
    for x in &v {
        if x == 2 {
            println!("two");
        }
    }
}
