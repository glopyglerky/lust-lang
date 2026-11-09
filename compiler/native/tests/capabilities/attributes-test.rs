// capability: attributes: test, ignore, should_panic (`lust test` runs them at the crate root and in inline modules)
// expect: ok
// command: test

#[test]
fn adds() {
    assert_eq!(1 + 1, 2);
}

#[test]
#[ignore]
fn slow() {
    panic!("ignored tests do not run");
}

#[test]
#[should_panic(expected = "boom")]
fn panics() {
    panic!("boom");
}

#[cfg(test)]
mod tests {
    #[test]
    fn in_module() {
        assert_eq!(2 * 2, 4);
    }

    mod nested {
        #[test]
        #[should_panic]
        fn deep() {
            panic!("nested");
        }
    }
}

fn main() {}
