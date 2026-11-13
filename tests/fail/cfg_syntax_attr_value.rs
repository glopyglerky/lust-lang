// expect: L0001
// The attribute grammar runs in cfg-disabled code: `= value` needs one expression.
#[cfg(test)]
mod tests {
    /// Doc is fine here.
    #[my_attr = 1 2]
    fn f() {}
}
fn main() { println!("ok"); }
