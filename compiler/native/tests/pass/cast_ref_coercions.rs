// Reference-to-reference `as` casts are coercion casts in rustc:
// array to slice, `&mut` to `&`, `&Vec` to slice, `&String` to `&str`.
fn main() {
    let a = [1u8, 2];
    let s = &a as &[u8];
    println!("{}", s.len());

    let v = vec![1u8, 2, 3];
    let s = &v as &[u8];
    println!("{}", s.len());

    let text = String::from("hi");
    let t = &text as &str;
    println!("{}", t.len());

    let mut m = [1u8];
    let s = &mut m as &mut [u8];
    s[0] = 9;
    println!("{}", m[0]);

    let mut n = [5u8];
    let s = &mut n as &[u8];
    println!("{}", s[0]);
}
