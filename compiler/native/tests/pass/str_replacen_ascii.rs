// str::replacen (capacity 32, first n matches, char and empty patterns), is_ascii, eq_ignore_ascii_case (R3).
fn main() {
    let s = String::from("aXaXa");
    for n in [0, 1, 2, 5, usize::MAX] {
        let r = s.replacen("a", "<>", n); println!("{r} {}", r.capacity());
        let r = s.replacen('X', "", n); println!("{r} {}", r.capacity());
        let r = "héllo".replacen("", "-", n); println!("{r} {}", r.capacity());
        let r = "aaaa".replacen("aa", "b", n); println!("{r} {}", r.capacity());
    }
    let long = "x".repeat(40); let r = long.replacen("x", "yy", 30); println!("{} {}", r.len(), r.capacity());
    let t = String::from("é"); println!("{} {} {}", "abc".is_ascii(), t.is_ascii(), "".is_ascii());
    println!("{} {} {}", "HeLLo".eq_ignore_ascii_case("hello"), "é".eq_ignore_ascii_case("É"), s.eq_ignore_ascii_case(&String::from("AxAxA")));
    let parts: Vec<String> = ["a.ts", "b.ts"].iter().map(|f| "x a.ts y".replacen("a.ts", f, 1)).collect(); println!("{parts:?}");
}
