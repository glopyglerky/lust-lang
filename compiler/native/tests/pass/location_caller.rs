// `std::panic::Location::caller()` as a plain value (no
// `#[track_caller]` fn in between): stored in a struct, passed,
// copied, compared, and measured. Line and column numbers are this
// file's, as rustc computes them for the same file.
use std::panic::Location;

struct Marked {
    loc: &'static Location<'static>,
    tag: i64,
}

fn show(m: &Marked) -> (u32, i64) {
    (m.loc.line(), m.tag)
}

fn main() {
    let a = Location::caller();
    let b = core::panic::Location::caller();
    let m = Marked { loc: Location::caller(), tag: 7 };
    let (mline, tag) = show(&m);
    let copied = *a;
    println!("{} {} {}", a.line(), b.line(), mline);
    println!("{} {} {}", a.column(), b.column(), m.loc.column());
    println!("{} {} {}", a.file() == file!(), a.file().ends_with("location_caller.rs"), tag);
    println!("{} {} {}", copied.line() == a.line() && copied == *a, a == b, a < b);
    println!("{} {}", std::mem::size_of::<Location>(), std::mem::align_of::<Location>());
}
