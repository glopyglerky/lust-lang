// A Display impl that returns Err inside a panic message ends the text
// there: std ignores the error while it formats the payload.
// panic: a x
struct S;
impl std::fmt::Display for S {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("x")?;
        Err(std::fmt::Error)
    }
}
fn main() {
    println!("start");
    panic!("a {} b {}", S, 5);
}
