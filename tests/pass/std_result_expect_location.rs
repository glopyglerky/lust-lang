// panic: std_result_expect_location.rs:12:10
// A std source method (Result::expect) reports its caller's location,
// as core's #[track_caller] does, with core's message.
fn parse(s: &str) -> Result<u8, String> {
    if s == "7" { Ok(7) } else { Err(format!("bad {}", s)) }
}

fn main() {
    let a = parse("7").expect("first");
    println!("{}", a);
    let b = parse("x")
        .expect("second");
    println!("{}", b);
}
