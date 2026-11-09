// A closure or char-set pattern read from an array element, a tuple
// field, or a struct field (was a wrong result: the pattern type was
// lost through the projection).
struct S<F> {
    f: F,
}
fn main() {
    let fs = [|c: char| c == 'a'];
    println!("{:?} {}", "ab".find(fs[0]), "ab".replace(fs[0], "x"));
    let t = (|c: char| c == 'a', 1);
    println!("{:?} {:?}", "ab".contains(t.0), "xab".strip_prefix(t.0));
    let s = S { f: |c: char| c == 'b' };
    println!("{:?} {}", "ab".rfind(s.f), "abb".trim_end_matches(s.f));
    let sets = [['a', 'b']];
    println!("{:?} {}", "cab".find(sets[0]), "cab".replacen(sets[0], "-", 1));
}
