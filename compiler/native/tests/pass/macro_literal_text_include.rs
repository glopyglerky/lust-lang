// Literal tokens keep their written text in a macro defined in another
// file: `stringify!` prints it, not the value, and not bytes of this file.
include!("macro_literal_text_include.rs.inc");
fn main() {
    println!("{}", sx!());
    println!("{} {}", one!(0x1), one!(1));
}
