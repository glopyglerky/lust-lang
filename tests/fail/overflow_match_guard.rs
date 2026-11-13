// expect: L0540
fn main() {
    let a: u8 = 255;
    let v: i32 = 1;
    match v {
        x if x == 1 => {
            let _b = a + 1;
        }
        _ => {}
    }
}
