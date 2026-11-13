// capability: attributes: test, ignore, should_panic (`lust test` uses native crate resolution, including file modules and macro-generated tests)
// expect: ok
// command: test

#[path = "mods/helper.rs"]
mod helper;

#[test]
fn uses_helper() {
    assert_eq!(helper::value(), 7);
}

fn main() {}
