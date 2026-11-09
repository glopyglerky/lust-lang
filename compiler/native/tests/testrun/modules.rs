// `lust test` runs #[test] fns in inline modules, at any depth, with the
// module path as the test name (as rustc --test does).
fn double(x: u32) -> u32 {
    x * 2
}

#[test]
fn top() {
    assert_eq!(double(1), 2);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_in_private_module() {
        assert_eq!(double(2), 4);
    }

    #[test]
    fn fails() {
        assert_eq!(double(2), 5);
    }

    #[test]
    #[ignore]
    fn ignored() {
        panic!("never runs");
    }

    mod nested {
        #[test]
        #[should_panic(expected = "deep")]
        fn deep_panic() {
            panic!("deep");
        }

        #[test]
        fn top() {}
    }
}

mod other { #[test] fn one_line() { assert!(1 + 1 == 2); } }

fn main() {}
