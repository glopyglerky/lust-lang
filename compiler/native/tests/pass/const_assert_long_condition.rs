// A no-message assert! whose condition rustc prints over several lines
// (longer than its 78-byte margin) or that holds an item is accepted in a
// const context: the text is never printed there (a true condition passes,
// a false one is a compile error). ts-rust's fswatch/unix.rs has the
// const _ form. In runtime code it stays L0809 (tests/reject).
#![allow(dead_code)]
const O_RDONLY: i32 = 0x0;
const O_CLOEXEC: i32 = 0x80000;
const O_NONBLOCK: i32 = 0x800;
const O_NOCTTY: i32 = 0x100;
const IN_NONBLOCK: i32 = 0x800;
const IN_CLOEXEC: i32 = 0x80000;
const _: () = assert!(
    O_RDONLY == 0x0
        && O_CLOEXEC == 0x80000
        && O_NONBLOCK == 0x800
        && O_NOCTTY == 0x100
        && IN_NONBLOCK == 0x800
        && IN_CLOEXEC == 0x80000
);
static S: () = assert!(O_RDONLY == 0x0 && O_CLOEXEC == 0x80000 && O_NONBLOCK == 0x800 && O_NOCTTY == 0x100);
const CHECK: () = assert!({ const K: i32 = 0x800; K == O_NONBLOCK });
macro_rules! check { ($e:expr) => { assert!($e) }; }
const _: () = check!(matches!(O_NOCTTY, 0x100) && stringify!(a == b).len() == 6 && concat!("x", "y").len() == 2 && O_RDONLY == 0);
struct T;
impl T {
    const OK: () = assert!(IN_NONBLOCK == O_NONBLOCK && IN_CLOEXEC == O_CLOEXEC && O_RDONLY == 0x0 && O_NOCTTY == 0x100);
}
const N: usize = {
    assert!(O_RDONLY == 0x0 && O_CLOEXEC == 0x80000 && O_NONBLOCK == 0x800 && O_NOCTTY == 0x100 && IN_NONBLOCK == 0x800);
    3
};
fn main() {
    const _: () = assert!(O_RDONLY == 0x0 && O_CLOEXEC == 0x80000 && O_NONBLOCK == 0x800 && O_NOCTTY == 0x100);
    let _ = (&S, CHECK, T::OK);
    println!("ok {}", [0u8; N].len());
}
