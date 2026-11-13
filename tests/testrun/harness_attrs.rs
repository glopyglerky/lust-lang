// Tests for `lust test` harness attributes: #[ignore] never runs its
// test, and #[should_panic] passes only on a panic with the expected
// text. Results must match `rustc --test` test by test.
#[test]
fn test_ok() {
    assert_eq!(1 + 1, 2);
}

#[test]
#[ignore]
fn test_ignored_fails() {
    panic!("this body never runs");
}

#[test]
#[ignore = "needs a fixture file"]
fn test_ignored_reason() {}

#[test]
#[should_panic]
fn test_should_panic() {
    panic!("boom");
}

#[test]
#[should_panic(expected = "boom")]
fn test_should_panic_text() {
    panic!("boom goes it");
}

#[test]
#[should_panic = "boom"]
fn test_should_panic_eq() {
    panic!("boom");
}

#[test]
#[should_panic]
fn test_should_panic_no_panic() {}

#[test]
#[should_panic(expected = "missing text")]
fn test_should_panic_wrong_text() {
    panic!("boom");
}
