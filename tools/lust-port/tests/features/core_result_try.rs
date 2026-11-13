// feature: core.result_try
// form: Result with ? and a From error conversion, matched at the top
#[derive(Debug)]
struct ParseErr;
#[derive(Debug)]
struct TopErr;
impl From<ParseErr> for TopErr { fn from(_: ParseErr) -> TopErr { TopErr } }
fn parse(ok: bool) -> Result<i32, ParseErr> { if ok { Ok(3) } else { Err(ParseErr) } }
fn run() -> Result<i32, TopErr> { let v = parse(true)?; Ok(v + 1) }
fn main() {
    let a = match run() { Ok(v) => v, Err(_) => 0 };
    let b = match parse(false) { Ok(v) => v, Err(_) => -1 };
    println!("{} {}", a, b);
}
