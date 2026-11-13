// `Location::caller()` is a const fn, so a user const fn may call it; at
// runtime it gives the location inside `here` (fx-attrs, tf-v07).
use std::panic::Location;

const fn here() -> &'static Location<'static> {
    Location::caller()
}

fn main() {
    println!("{} {}", here().line(), here().column());
}
