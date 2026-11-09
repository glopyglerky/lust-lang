fn main() {
    let values: Vec<_> = (1..=6).collect();
    assert_eq!(values.iter().sum::<i32>(), 21);
    struct Guard<'a>(&'a std::cell::Cell<u32>);
    impl Drop for Guard<'_> {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let drops = std::cell::Cell::new(0);
    let recovered = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = Guard(&drops);
        panic!("expected");
    }));
    assert!(recovered.is_err());
    assert_eq!(drops.get(), 1);
    println!("sum=21; unwind=recovered; drops=1");
}

#[test]
fn runtime_and_cleanup() {
    main();
}
