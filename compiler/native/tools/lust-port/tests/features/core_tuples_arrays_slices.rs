// feature: core.tuples_arrays_slices
// form: tuple value and field, array repeat, slice param and range index
fn head(xs: &[i32]) -> i32 { xs[0] }
#[allow(dead_code)]
fn fail() -> ! { panic!("x") }
fn main() {
    let t = (3, "x");
    let a = [7; 3];
    let v = vec![5, 6, 7, 8];
    let w: Vec<_> = v.iter().copied().collect();
    println!("{} {} {} {} {}", t.0, a[2], head(&v), v[1..3].len(), w.len());
}
