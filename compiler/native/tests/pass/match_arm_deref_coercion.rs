fn pick<'a>(o: &'a Option<String>, d: &'a str) -> &'a str {
    match o {
        Some(s) => s,
        None => d,
    }
}
fn main() {
    let o = Some(String::from("t"));
    let r: &str = match &o {
        Some(s) => s,
        None => "none",
    };
    let q: &str = if let Some(s) = &o { s } else { "" };
    println!("{} {}", r, q);
    let e: Option<String> = None;
    println!("{}", pick(&e, "def"));
    println!("{}", pick(&o, "def"));
    let v = vec![1u8, 2, 3];
    let s: &[u8] = if v.len() > 1 { &v } else { &[9] };
    println!("{}", s.len());
    let arr = [4u8, 5];
    let s2: &[u8] = match arr.len() {
        2 => &arr,
        _ => &[],
    };
    println!("{}", s2.len());
}
