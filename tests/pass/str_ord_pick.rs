// `max`, `min`, and `clamp` on `&str` compare the text.
fn main() {
    let words = ["pear", "apple", "fig"];
    let mut hi = "";
    let mut lo = "zzz";
    for w in words {
        hi = hi.max(w);
        lo = lo.min(w);
    }
    println!("{} {}", hi, lo);
    println!("{} {} {}", "m".clamp("a", "k"), "b".clamp("c", "k"), "d".clamp("c", "k"));
    println!("{:?}", Some("x").max(Some("y")));
}
