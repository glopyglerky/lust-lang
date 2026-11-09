// feature: iterators.chains
// form: for over an array, a Vec by value, a &Vec and a range; collect to String
fn main() {
    let mut total = 0;
    for x in [1, 2, 3] { total += x; }
    let v = vec![10, 20];
    for x in v { total += x; }
    let w = vec![4, 5];
    for x in &w { total += *x; }
    for i in 0..3 { total += i; }
    let s: String = "ab".chars().collect();
    println!("{} {}", total, s.len());
}
