// expect: L0702
// Valid Rust: #[track_caller] on a trait impl method. A dyn call could not pass the location, so lust rejects it (W-CHECK-G3).
trait T { fn at(&self) -> u32; }
struct S;
impl T for S {
    #[track_caller]
    fn at(&self) -> u32 { std::panic::Location::caller().line() }
}
fn main() { println!("{}", S.at() > 0); }
