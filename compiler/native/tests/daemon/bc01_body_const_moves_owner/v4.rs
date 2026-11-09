// A body const's value folds into its owner fn's source: a const fn edit
// that moves it rechecks the owner (and its instances), while generic
// reuse stays on for the rest.
const fn base() -> usize {
    2 + 2
}

fn sized<T: Default + Copy>(x: T) -> usize {
    const N: usize = base() * 2;
    let a = [x; N];
    a.len()
}

fn plain() -> usize {
    const M: usize = base() + 1;
    M
}

fn other<T: Copy>(x: T) -> (T, T) {
    (x, x)

}

fn main() {
    println!("{} {} {:?}", sized(1u8), plain(), other(5));
}
