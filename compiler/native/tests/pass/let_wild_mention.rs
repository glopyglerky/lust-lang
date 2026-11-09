// `let _ = place;` only mentions the place (rustc `PlaceMention`): it does
// not read, move, or need it initialized, so a mutable borrow of it may
// stay live across the `let`.
fn main() {
    let mut a = 1;
    let r = &mut a;
    let _ = a;
    *r = 2;
    let mut p = (1, 2);
    let q = &mut p;
    let _ = p.0;
    q.0 = 3;
    let s = String::from("x");
    drop(s);
    let _ = s;
    let u: String;
    let _ = u;
    let v = vec![String::from("a")];
    let _ = v[0];
    println!("{} {} {}", a, p.0, v.len());
}
