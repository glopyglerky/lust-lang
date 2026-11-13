// The hidden type of `impl Trait + 's` may hold a `T` that a bound (`T: 's`, `T: 'b` with `'b: 's`,
// `T: 'static`) or an implied bound (`&'s T`) makes outlive `'s`.
fn a<'s, T: 's>(t: T) -> impl Sized + 's { t }
fn b<'s, 'b: 's, T: 'b>(t: T, q: &'s u8) -> impl Sized + 's { let _ = q; (t, 1u8) }
fn c<'s, T: 'static>(t: T) -> impl Sized + 's { vec![t] }
fn d<'s, T>(t: &'s T) -> impl Sized + 's { t }
fn e<'s, T: Copy + 's>(t: T) -> impl Fn() -> T + 's { move || t }
fn main() {
    let x = 3u8;
    let _ = (a(1u8), b(2u8, &x), c(3u8), d(&x));
    println!("{}", e(4u8)());
}
