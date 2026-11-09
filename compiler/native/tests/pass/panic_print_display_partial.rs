// A Display impl's own writes reach the LineWriter one by one: "start xa\n"
// goes out, the held "b1" is lost at the abort. eprint! is unbuffered.
// panic: in fmt
struct S;
impl std::fmt::Display for S {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a\nb{}", 1)?;
        panic!("in fmt")
    }
}
fn main() {
    print!("start ");
    eprint!("e1 {} ", 2);
    eprintln!("e2 {}", S);
}
