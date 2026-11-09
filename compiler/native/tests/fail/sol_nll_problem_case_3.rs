// expect: L0303
// Sol review probe `nll_problem_case_3`; rustc: E0502 cannot borrow `*v` as mutable because it is also borrowed as immutable
fn first_or_push(v: &mut Vec<i32>) -> &i32 {
    if let Some(x) = v.first() { return x; }
    v.push(1);
    &v[0]
}
fn main() { let mut v = Vec::new(); println!("{}", first_or_push(&mut v)); }
