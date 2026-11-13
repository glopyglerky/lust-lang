// feature: core.patterns_control
// form: let-else, if-let, labeled break with a value, struct update, slice pattern with @
#[derive(Clone)]
struct Opts { strict: bool, level: i32 }
fn need(x: Option<i32>) -> i32 { let Some(v) = x else { return -1 }; v }
fn main() {
    let base = Opts { strict: true, level: 1 };
    let next = Opts { level: 2, ..base };
    let found = 'outer: loop { break 'outer 42; };
    let xs = [1, 2, 3];
    let mid = match xs { [_, rest @ .., _] => rest.len(), _ => 0 };
    let (first, ..) = (8, 9, 10);
    let Opts { level, .. } = next;
    println!("{} {} {} {}", level, found, mid, need(Some(5)) + first);
}
