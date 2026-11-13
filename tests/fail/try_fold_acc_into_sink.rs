// expect: L0306
// rustc E0597: a later round's accumulator is the captured `s`, and the closure stores every
// accumulator in `out`, which outlives `s`.
fn main() {
    let v = vec![1u32, 2, 3];
    let mut out: Vec<&str> = Vec::new();
    {
        let s = String::from("z");
        let r = v.iter().try_fold("x", |a, _| {
            out.push(a);
            Some(s.as_str())
        });
        println!("{:?}", r);
    }
    println!("{:?}", out);
}
