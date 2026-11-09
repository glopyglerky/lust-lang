// Patterns through type aliases of Option and Result (`OP::Some(x)`,
// `OP::None`, `RR::Ok(x)`, `RR::Err(e)`), in match, if-let, and let-else.
type OP = Option<u8>;
type GO<T> = Option<T>;
type RR = Result<u16, String>;

fn show(o: OP) -> u8 {
    match o {
        OP::Some(x) => x,
        OP::None => 0,
    }
}

fn first(r: RR) -> u16 {
    let RR::Ok(v) = r else { return 99 };
    v
}

fn main() {
    println!("{} {}", show(Some(4)), show(None));
    let g: Option<&str> = Some("g");
    if let GO::Some(s) = g {
        println!("{s}");
    }
    let r: RR = Err(String::from("e"));
    match &r {
        RR::Ok(v) => println!("ok {v}"),
        RR::Err(e) => println!("err {e}"),
    }
    println!("{} {}", first(Ok(7)), first(r));
}
