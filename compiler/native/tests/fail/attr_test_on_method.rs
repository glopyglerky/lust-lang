// expect: L0402
// rustc: the `test` attribute may only be used on a free function, also without --test.
struct S;

impl S {
    #[test]
    fn t() {}
}

fn main() {}
