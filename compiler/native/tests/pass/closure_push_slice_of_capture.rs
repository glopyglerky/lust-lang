// A closure stores slices of its captured `s` into its captured `out`.
fn main() {
    let s = String::from("hello");
    let mut out: Vec<&str> = Vec::new();
    let mut add = |n: usize| out.push(&s[..n]);
    add(1);
    add(2);
    println!("{:?}", out);
}
