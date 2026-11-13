// expect: L0306
// rustc E0597: a `&'static str` field accepts only `'static` data (ARCH 5.1 step A).
struct S {
    name: &'static str,
}

fn main() {
    let s = String::from("hi");
    let x = S { name: &s };
    println!("{}", x.name);
}
