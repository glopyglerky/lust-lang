// A body edit that removes bytes moves the std methods spliced after
// the user text. Every span after the edit must move with them, the
// span of each std impl too: one that stayed put fell outside the std
// method range and was rejected as a user impl of `Result` (L0102).
fn total(xs: &[i64]) -> i64 {
    let mut t = 0;
    for x in xs {
        t += x;
    }
    t
}
fn main() {
    let r: Result<i64, String> = Ok(total(&[1, 2, 3]));
    let o: Option<i64> = r.ok().map(|v| v * 2);
    println!("{}", o.unwrap_or(0));
}
