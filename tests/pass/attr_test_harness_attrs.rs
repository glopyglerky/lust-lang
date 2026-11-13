// The test harness attributes in a normal build: the #[test] fns are
// removed, and on a plain fn rustc only warns, so lust accepts and
// ignores them. `lust test` honors them (tests/testrun).
#[test]
#[ignore]
fn skipped() {
    panic!("never runs");
}
#[test]
#[ignore = "writes a snapshot file"]
fn skipped_with_reason() {}
#[test]
#[should_panic]
fn panics() {
    panic!("boom");
}
#[test]
#[should_panic(expected = "boom")]
fn panics_with_text() {
    panic!("boom goes it");
}
#[test]
#[should_panic = "boom"]
fn panics_eq_form() {
    panic!("boom");
}
#[ignore]
fn plain() -> i64 {
    7
}
fn main() {
    println!("{}", plain());
}
