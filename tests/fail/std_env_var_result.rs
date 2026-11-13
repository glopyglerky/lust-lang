// expect: L0110
// var returns Result<String, VarError>.
fn main() {
    let s: String = std::env::var("HOME");
}
