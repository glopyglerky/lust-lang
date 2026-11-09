// expect: L0540
const SEVEN: i32 = 7;
fn main() {
    let a: u8 = 255;
    let v: i32 = 7;
    match v {
        SEVEN => {
            let _b = a + 1;
        }
        _ => {}
    }
}
