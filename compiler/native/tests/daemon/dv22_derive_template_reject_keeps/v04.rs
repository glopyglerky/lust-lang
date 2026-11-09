// A rejected derive edit whose only error is in a template body (a generic fn clones the type): the derive delta undoes and keeps the program, so the revert takes the delta (was a full collect, kept=miss:none).
const ROUTE: u8 = 1;
#[derive(Clone)]
pub struct It { v: Vec<u32>, i: usize }
fn twice<T: std::fmt::Debug>(t: T, it: &It) -> (It, T) { (it.clone(), t) }
fn plain(it: &It) -> usize { it.v.len() + it.i }
fn main() {
    let it = It { v: vec![1, 2], i: 0 };
    let (c, n) = twice(ROUTE, &it);
    println!("{:?} {} {}", c, n, plain(&it));
}
