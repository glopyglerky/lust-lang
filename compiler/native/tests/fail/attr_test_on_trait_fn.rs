// expect: L0402
// rustc: the `test` attribute may only be used on a free function, also on a trait fn.
trait T {
    #[test]
    fn t() {}
}

fn main() {}
