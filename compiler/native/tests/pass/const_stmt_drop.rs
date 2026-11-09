// A const value as an expression statement is a temporary: it drops
// at the end of the statement.
struct D(i32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
const CD: D = D(1);

fn main() {
    CD;
    println!("a");
    const { D(2) };
    println!("b");
}
