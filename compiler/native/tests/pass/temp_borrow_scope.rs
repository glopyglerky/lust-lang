// Temporary borrows that end with the statement (or are never used) are valid.
fn make() -> String {
    String::from("temp")
}

fn g(v: &mut Vec<i64>) {
    v.push(1);
    println!("{}", v.len());
}

fn main() {
    // Borrowed from a temporary, but never used: rustc accepts this.
    let _s: &str = make().as_str();
    println!("done");
    // A temporary passed straight into a call lives for the whole statement.
    g(&mut Vec::new());
    // A holder declared before the value it borrows, used before the scope ends.
    let a = String::from("a");
    let mut r = &a;
    let b = String::from("b!");
    r = &b;
    println!("{}", r);
    // Direct borrow of a temporary in a let: the temporary lives for the block.
    let v: &Vec<i64> = &Vec::new();
    println!("{}", v.len());
}
