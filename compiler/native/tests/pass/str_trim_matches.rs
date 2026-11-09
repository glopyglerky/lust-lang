// `trim_matches`, `trim_start_matches`, and `trim_end_matches`
// over char, char-set, closure, and `&str` patterns (tf-str).
use std::cell::RefCell;

fn main() {
    println!("[{}] [{}] [{}]", "xxabxx".trim_matches('x'), "xxabxx".trim_start_matches('x'), "xxabxx".trim_end_matches('x'));
    println!("[{}]", "xyabxy".trim_matches(&['x', 'y'][..]));
    println!("[{}] [{}]", "abcabcab".trim_start_matches("abc"), "abcabcab".trim_end_matches("ab"));
    println!("[{}] [{}]", "aaabaaa".trim_start_matches("aa"), "aaabaaa".trim_end_matches("aa"));
    println!("[{}] [{}]", "ab".trim_start_matches(""), "ab".trim_end_matches(""));
    println!("[{}]", "  ab  ".trim_matches(|c: char| c == ' '));
    println!("[{}] [{}]", "ééabé".trim_matches('é'), "ééabé".trim_start_matches(['é']));
    println!("[{}] [{}] [{}]", "".trim_matches('x'), "xxx".trim_matches('x'), "abc".trim_matches(['x'; 0]));
    let s = String::from("..hi..");
    println!("[{}] [{}]", s.trim_matches('.'), s.trim_end_matches(".."));
    // `trim_matches` scans the front first, then the back, each
    // char once and stopping at the first non-match.
    let log = RefCell::new(String::new());
    let r = "xxabxx".trim_matches(|c: char| {
        log.borrow_mut().push(c);
        c == 'x'
    });
    println!("[{}] {}", r, log.borrow());
    let log2 = RefCell::new(String::new());
    let r = "xxab".trim_end_matches(|c: char| {
        log2.borrow_mut().push(c);
        c == 'x'
    });
    println!("[{}] {}", r, log2.borrow());
}
