include!(concat!(env!("OUT_DIR"), "/gen.rs"));

fn main() {
    println!("{}", generated());
    println!("{}", env!("GREETING"));
    println!("{}", cfg!(has_gen));
    println!("{}", include_str!("data.txt").trim());
    println!("{}", oddlib::lib_text());
    println!("{}", env!("CARGO_MANIFEST_DIR").ends_with("sp ace#é"));
}
