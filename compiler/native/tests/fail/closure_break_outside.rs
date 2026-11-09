// expect: L0120
fn main() {
    let mut i = 0;
    loop {
        let c = || { break; };
        c();
        i += 1;
    }
}
