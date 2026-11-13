// include_str!/include_bytes! with a concat! path inside a macro_rules
// body: concat! and the literal fragment expand first, then the path
// resolves relative to this file.
macro_rules! load {
    ($name:literal) => {
        include_str!(concat!("macro_", $name))
    };
}
macro_rules! load_bytes {
    ($name:literal) => {
        include_bytes!(concat!("macro_", $name))
    };
}
fn main() {
    let text = load!("data.txt");
    println!("text='{}' bytes={}", text.trim(), load_bytes!("data.txt").len());
}
