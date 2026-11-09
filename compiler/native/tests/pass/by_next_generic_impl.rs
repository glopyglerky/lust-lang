// Adapters over a user iterator whose Iterator impl is generic:
// the expansion calls the impl's `next` instance for `G<i32>`.
struct G<T>(Vec<T>);

impl<T> Iterator for G<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.0.pop()
    }
}

fn main() {
    let v: Vec<i32> = G(vec![1, 2]).map(|x| x * 2).collect();
    println!("{v:?}");
}
