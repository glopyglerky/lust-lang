// An empty brace group stringifies as `{}`, with no space inside.
fn main() {
    println!("{}", stringify!(fn f() { }));
    println!("{}", stringify!({ }));
}
