// feature: macros.std_unsupported
// form: include_bytes! of a sibling file, as in ts-rust embedded data
fn main() {
    let bytes = include_bytes!("imp_helper.rs");
    println!("{}", bytes.len());
}
