// `Option::as_slice` and `as_mut_slice`, `String::as_mut_str`,
// `Option<String>` and `Result<String, E>` `as_deref_mut`, and
// `as_deref` with a payload type fixed after the call
// (fx-option-rest).
fn main() {
    let a = Some(3);
    println!("{:?}", a.as_slice());
    let n: Option<i32> = None;
    println!("{:?} {}", n.as_slice(), n.as_slice().is_empty());
    let mut m = Some(4);
    m.as_mut_slice()[0] = 9;
    println!("{:?}", m);
    let mut e: Option<String> = None;
    println!("{}", e.as_mut_slice().len());

    let mut s: Option<String> = Some(String::from("ab"));
    if let Some(x) = s.as_deref_mut() {
        let y: &mut str = x;
        println!("{} {}", y, y.len());
    }
    println!("{:?}", s);
    let mut r: Result<String, i32> = Ok(String::from("cd"));
    if let Ok(x) = r.as_deref_mut() {
        let y: &mut str = x;
        println!("{} {}", y, y.len());
    }
    println!("{:?}", r);

    let mut t = String::from("ef");
    let u: &mut str = t.as_mut_str();
    println!("{} {}", u, u.len());

    // The payload type is fixed after the `as_deref` call.
    let mut x = None;
    let was_some = x.as_deref().is_some();
    x = Some(String::from("a"));
    println!("{} {:?}", was_some, x);
    let mut y = None;
    let len = y.as_deref().map_or(0, |v: &[i32]| v.len());
    y = Some(vec![1, 2]);
    println!("{} {:?}", len, y.as_deref());

    let answer = 42;
    let from = std::slice::from_ref(&answer);
    let mut single = 1;
    let from_mut = std::slice::from_mut(&mut single);
    from_mut[0] = 2;
    println!("{:?} {}", from, single);
}
