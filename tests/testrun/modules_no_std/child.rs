#[path = "child/nested.rs"]
mod nested;

#[test]
fn original_source_paths() {
    assert!(file!().ends_with("modules_no_std/child.rs"));
    assert_eq!(include_str!("data.txt"), "native fixture\n");
}

macro_rules! generated_test {
    () => {
        #[test]
        fn from_macro() {
            assert_eq!(2 + 3, 5);
        }
    };
}
generated_test!();

#[test]
#[ignore]
fn ignored() {
    panic!("ignored test must not execute");
}

#[test]
#[should_panic(expected = "module panic")]
fn expected_panic() {
    panic!("module panic");
}
