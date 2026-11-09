// Split predicates are called once per char in scan order (front and back
// mixed), and an owned closure drops with its iterator or after
// split_once (W-STD-B).
struct Loud(String);
impl Drop for Loud {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn main() {
    let mut calls = 0;
    let v: Vec<&str> = "a,b,c".split(|c: char| {
        calls += 1;
        c == ','
    })
    .collect();
    println!("{v:?} {calls}");
    let mut seen = Vec::new();
    let mut it = "x;y;z".split(|c: char| {
        seen.push(c);
        c == ';'
    });
    let back = it.next_back();
    let front = it.next();
    drop(it);
    println!("{back:?} {front:?} {seen:?}");
    let tag = Loud(String::from(":"));
    let parts: Vec<&str> = "p:q".split(move |c: char| tag.0 == c.to_string()).collect();
    println!("{parts:?}");
    let t2 = Loud(String::from("="));
    println!("{:?}", "k=v".split_once(move |c: char| t2.0 == c.to_string()));
    println!("{:?}", "ünï:cödé".split("ï:").collect::<Vec<_>>());
    println!("{:?}", "a\r\nb\r\n".lines().rev().collect::<Vec<_>>());
    let text = String::from("one two  three");
    let words: Vec<&str> = text.split(' ').filter(|w| !w.is_empty()).collect();
    println!("{words:?} {}", text.split_whitespace().count());
    println!("end");
}
