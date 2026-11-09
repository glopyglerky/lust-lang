// A fn whose body ends in a diverging call statement still has an
// unreachable return value of its own type. Result<(), String> is
// niche-filled (the String capacity niche), so its C union starts with
// the empty unit payload: the zero literal must be `{}`, not `{0}`.
fn die() -> ! {
    panic!("x")
}

fn f() -> Result<(), String> {
    die();
}

fn g() -> Result<((), u8), Vec<u8>> {
    die();
}

fn h(n: u8) -> Option<Result<(), String>> {
    if n > 200 {
        die();
    }
    Some(Ok(()))
}

fn main() {
    if std::env::args().count() > 5 {
        let _ = f();
        let _ = g();
    }
    println!("{:?}", h(1));
}
