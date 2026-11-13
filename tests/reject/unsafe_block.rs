// expect: L0404
fn main() {
    let x = 1;
    unsafe {
        let _y = x;
    }
    println!("{}", x);
}
