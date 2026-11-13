// `str::find` and `str::rfind` with &str and char patterns
// (W-BENCH4 rank 16), including empty patterns and multi-byte text.
fn main() {
    let s = String::from("a.b.c");
    println!("{:?} {:?}", s.rfind('.'), s.find('.'));
    println!("{:?} {:?}", s.find("b.c"), s.rfind("b.c"));
    println!("{:?} {:?}", s.find("x"), s.find(""));
    println!("{:?} {:?}", s.rfind(""), s.rfind("a.b.c"));
    println!("{:?} {:?}", s.find("a.b.c."), "".find(""));
    let u = "héllo wörld";
    println!("{:?} {:?}", u.find('ö'), u.rfind('l'));
    println!("{:?} {:?}", u.find("llo"), u.rfind("l"));
    let rep = "ababab";
    println!("{:?} {:?}", rep.find("ab"), rep.rfind("ab"));
    println!("{:?}", rep[rep.rfind('b').unwrap()..].len());
}
