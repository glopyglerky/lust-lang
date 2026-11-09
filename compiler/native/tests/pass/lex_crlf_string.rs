// A CRLF inside a string literal reads as LF, as rustc reads the file.
fn main() {
    let s = "a
b";
    let r = r"c
d";
    let t = "e\
    f";
    println!("{} {} {} {}", s.len(), r.len(), t, stringify!("g
h") == "\"g\nh\"");
}
