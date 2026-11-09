// expect: L0001
// rustc E0658: an attribute on an expression statement whose top level is a
// binary operation goes to the left operand, so even a cfg that is off is an
// error ("removing an expression is not supported in this position").
fn t(s: &str) -> i32 { println!("{s}"); 1 }
fn main() {
    #[cfg(any())]
    t("a") - t("b");
}
