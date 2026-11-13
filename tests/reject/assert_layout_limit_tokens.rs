// expect: L0809
// The bounded multiline assertion printer admits at most 512 tokens.
fn main() {
    let flag = true;
    assert!(flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag &&
        flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag &&
        flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag &&
        flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag &&
        flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag &&
        flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag &&
        flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag &&
        flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag &&
        flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag &&
        flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag &&
        flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag &&
        flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag &&
        flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag &&
        flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag &&
        flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag && flag);
}
