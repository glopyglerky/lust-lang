// capability: rejected: module files (`mod name;`) in `lust test`, which reads the one file it is given
// expect: L9001
// command: test

#[path = "mods/helper.rs"]
mod helper;

#[test]
fn uses_helper() {
    assert_eq!(helper::value(), 7);
}

fn main() {}
