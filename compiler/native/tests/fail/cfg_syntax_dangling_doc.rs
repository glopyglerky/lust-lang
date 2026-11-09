// expect: L0001
// A doc comment with no item after it is a syntax error in disabled code too.
#[cfg(test)]
mod tests {
    fn f() {}
    /// This documents nothing.
}
fn main() { println!("ok"); }
