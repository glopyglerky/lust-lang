// expect: L0507
// Two block enum globs that both bring `A` (rustc E0659 at the use).
fn main() {
    #[allow(dead_code)]
    enum E1 { A, B }
    #[allow(dead_code)]
    enum E2 { A, C }
    use E1::*;
    use E2::*;
    let _ = A;
}
