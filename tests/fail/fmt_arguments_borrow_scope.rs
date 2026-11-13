// expect: L0306
// An `Arguments` borrows its format arguments: a value dropped at
// the end of a block may not back one used after it.
fn main() {
    let a = {
        let s = String::from("temp");
        format_args!("val {}", s)
    };
    println!("{}", a);
}
