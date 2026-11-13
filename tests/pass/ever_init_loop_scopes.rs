// Locals declared in loop bodies start fresh each iteration, also on
// continue and break paths: deferred init and moves there are fine.
fn d(i: i32) -> String { format!("s{}", i) }
fn main() {
    let mut out = Vec::new();
    for i in 0..3 {
        let a = d(i);
        if i == 1 { out.push(a); continue; }
        let b;
        if i == 2 { b = d(9); drop(a); out.push(b); break; }
        b = a;
        out.push(b);
    }
    let mut k = 0;
    while k < 3 {
        let s = d(k);
        k += 1;
        match k { 1 => { drop(s); continue; } _ => out.push(s) }
    }
    for s in vec![d(1), d(2)] { let t = s; out.push(t); }
    loop {
        let x;
        x = d(5);
        out.push(x);
        if out.len() > 6 { break; }
    }
    let v: Vec<String> = out.into_iter().map(|s| { let z = s; z }).collect();
    let mut it = v.iter();
    while let Some(s) = it.next() { let c = s.clone(); drop(c); }
    let y: String;
    if v.len() > 2 { y = d(1); } else { y = d(2); }
    println!("{:?} {}", v, y);
}
