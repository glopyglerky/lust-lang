// An array meets an `IntoIterator` bound: selection knows core's
// array impl (and the slice impls behind `&[T; N]`), so the
// associated `IntoIter: Iterator` projection normalizes at a call.
fn collect_strings<I: IntoIterator>(d: I) -> Vec<String>
where
    I::Item: Into<String>,
{
    d.into_iter().map(|s| s.into()).collect()
}
fn count<I: IntoIterator>(d: I) -> usize {
    d.into_iter().count()
}
fn main() {
    println!("{:?}", collect_strings(["a", "b"]));
    println!("{}", count(&["c", "d"]));
    println!("{}", count([1, 2, 3]));
    println!("{}", count(&[4, 5]));
}
