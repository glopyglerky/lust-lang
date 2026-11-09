// One test never ends; the runner must time it out and still run the other.
#[test]
fn test_quick() {
    assert!(1 < 2);
}

#[test]
fn test_hangs() {
    loop {}
}
