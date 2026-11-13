use std::panic::Location;

#[track_caller]
pub fn here() -> &'static Location<'static> {
    Location::caller()
}

#[track_caller]
pub fn wrapper() -> &'static Location<'static> {
    here()
}

pub fn from_module() -> String {
    let l = here();
    format!("{}:{}:{} {}", l.file(), l.line(), l.column(), l)
}

pub fn through_closure() -> String {
    let f = || wrapper();
    let l = f();
    format!("{l}")
}

#[track_caller]
pub fn fail(why: &str) -> ! {
    let l = Location::caller();
    panic!("{why} at {}:{}", l.file(), l.line())
}
