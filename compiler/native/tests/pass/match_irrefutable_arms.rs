// Irrefutable arms (and the arms after them) still check and run as before.
fn main() {
    let v = 5;
    let s = String::from("a");
    let r = match v {
        x => x + 1,
        _ => 0,
    };
    let t = match &s {
        y => y.len(),
    };
    let o = Some(String::from("b"));
    let n = match o {
        Some(z) => z.len(),
        None => 0,
    };
    let mut w = vec![1];
    match w.len() {
        _ => w.push(2),
    }
    let q = match (1, String::from("c")) {
        (a, b) => a + b.len(),
        #[allow(unreachable_patterns)]
        _ => 9,
    };
    println!("{} {} {} {:?} {} {}", r, t, n, w, q, s);
}
