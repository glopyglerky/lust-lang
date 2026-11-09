// Tests for `lust test`: a mix of passing, failing, and panicking tests.
fn add(a: i64, b: i64) -> i64 {
    a + b
}

fn main() {
    println!("{}", add(1, 2));
}

#[test]
fn test_add() {
    assert_eq!(add(1, 2), 3);
    assert_eq!(add(-1, 1), 0);
}

#[test]
fn test_vec() {
    let mut v: Vec<i64> = Vec::new();
    v.push(10);
    v.push(20);
    assert_eq!(v.len(), 2);
    assert_eq!(v[0] + v[1], 30);
    println!("vec ok");
}

#[test]
fn test_fails() {
    assert_eq!(add(1, 1), 3);
}

#[test]
fn test_panics() {
    panic!("boom");
}

#[test]
fn test_string() {
    let mut s = String::from("a");
    s.push_str("b");
    assert_eq!(s, "ab");
}
