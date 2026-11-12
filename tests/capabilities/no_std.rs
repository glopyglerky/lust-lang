// capability: attributes: no_std (core prelude; root `extern crate std;` and `extern crate alloc;` declarations)
// expect: ok
#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;
extern crate std;

fn main() {
    let values = alloc::vec![3u8, 5, 8];
    let answer: Result<u8, ()> = Some(values[1]).ok_or(());
    assert_eq!(answer, Ok(5));
    std::println!("{}", answer.unwrap());
}
