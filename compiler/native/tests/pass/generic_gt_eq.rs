// A `>` that closes generic arguments directly before `=` lexes
// as one `>=` token; the parser splits it, as rustc does.
use std::panic::Location;

const C: Option<Vec<u8>> = None;
static S: Option<Vec<u16>> = None;

fn main() {
    let l: &'static Location<'static> = Location::caller();
    let v: Vec<Vec<u8>> = vec![vec![7]];
    println!("{} {} {} {}", l.line(), v[0][0], C.is_none(), S.is_none());
}
