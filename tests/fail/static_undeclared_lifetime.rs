// expect: L0203
// rustc E0261: a static's type names a lifetime nothing declares
// (parse-fuzz seed 4 found `&'statuseic [N]`).
static KEEP: std::sync::Mutex<Vec<&'statuseic [u8]>> = std::sync::Mutex::new(Vec::new());
fn main() {
    KEEP.lock().unwrap().push(&[1]);
}
