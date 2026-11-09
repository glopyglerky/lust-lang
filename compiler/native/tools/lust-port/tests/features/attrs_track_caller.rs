// feature: attrs.track_caller
// form: track_caller fns reading Location::caller(), as ts-rust's go_panic records its caller
use std::panic::Location;
struct Report { at: &'static Location<'static>, msg: String }
#[track_caller]
fn report(msg: &str) -> Report { Report { at: Location::caller(), msg: msg.to_string() } }
#[track_caller]
fn here() -> u32 { Location::caller().line() }
fn main() {
    let r = report("boom");
    println!("{} {} {}", r.msg, r.at.line(), r.at.column());
    println!("{}", here() > 0);
}
