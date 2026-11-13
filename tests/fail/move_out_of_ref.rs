// expect: L0305
fn first(v: &Vec<String>) -> String {
    let s = v[0];
    s
}

fn main() {
    println!("{}", first(&vec![String::from("a")]));
}
