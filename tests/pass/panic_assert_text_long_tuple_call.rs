// Calling an unnamed tuple field has both a Field and a Call layout box.
// panic: assertion failed:
struct Record { first: i64, second: i64 }
impl Record {
    fn functions(self) -> (fn(i64) -> bool,) {
        let _ = self.first + self.second;
        (reject,)
    }
}
fn reject(_: i64) -> bool { false }
fn main() {
    let first_long_argument_name = 1;
    let second_long_argument_name = 2;
    assert!(Record { first: first_long_argument_name, second: second_long_argument_name }.functions().0(1));
}
