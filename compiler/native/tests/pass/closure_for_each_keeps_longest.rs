// A `for_each` closure stores an item into a captured `&str` local.
fn main() {
    let words = vec![String::from("a"), String::from("bb")];
    let mut longest: &str = "";
    words.iter().for_each(|w| if w.len() > longest.len() { longest = w; });
    println!("{}", longest);
}
